use rustc_hash::FxHashMap;
use swc_common::EqIgnoreSpan;
use swc_ecma_ast::*;
use swc_ecma_utils::ExprExt;

use crate::program_data::{ParameterValues, ProgramData, VarUsageInfo};

/// Track parameter values at function callsites.
pub(crate) fn track_params_at_callsite(
    data: &mut ProgramData,
    callee_id: &Id,
    args: &[ExprOrSpread],
    param_count: usize,
) {
    // Get function info
    let Some(fn_info) = data.vars.get(callee_id) else {
        return;
    };

    // Only track for actual functions
    if !fn_info.flags.contains(crate::program_data::VarUsageInfoFlags::DECLARED_AS_FN_DECL)
        && !fn_info.flags.contains(crate::program_data::VarUsageInfoFlags::DECLARED_AS_FN_EXPR)
    {
        return;
    }

    // Track each parameter value
    for param_idx in 0..param_count {
        let arg_value = args
            .get(param_idx)
            .map(|arg| {
                if arg.spread.is_none() {
                    Some(&*arg.expr)
                } else {
                    None
                }
            })
            .flatten()
            .or_else(|| Some(&Expr::Ident(Ident::new("undefined".into(), Default::default())) as &Expr));

        if let Some(value) = arg_value {
            // Only track simple, side-effect-free values
            if !is_trackable_value(value) {
                continue;
            }

            // Get parameter ID (we'll need to correlate this with function parameters)
            // For now, we'll store this in a temporary map and process it later
            let param_key = (callee_id.clone(), param_idx);
            track_param_value(data, param_key, value);
        }
    }
}

/// Track a parameter value for a specific parameter position.
fn track_param_value(data: &mut ProgramData, param_key: (Id, usize), value: &Expr) {
    // This is a placeholder - we need to store this mapping somewhere
    // and then correlate it with actual parameter identifiers later
}

/// Check if a value is suitable for tracking (side-effect free and not too large).
fn is_trackable_value(expr: &Expr) -> bool {
    match expr {
        // Literals are trackable
        Expr::Lit(lit) => {
            match lit {
                // Don't track large strings
                Lit::Str(s) if s.value.len() > 32 => false,
                _ => true,
            }
        }
        // Identifier 'undefined' is trackable
        Expr::Ident(i) if &*i.sym == "undefined" => true,
        // Simple unary expressions
        Expr::Unary(UnaryExpr { op: op!("!"), arg, .. }) => {
            matches!(&**arg, Expr::Lit(_))
        }
        Expr::Unary(UnaryExpr { op: op!("-"), arg, .. }) => {
            matches!(&**arg, Expr::Lit(Lit::Num(_)))
        }
        // Other expressions are not trackable for now
        _ => false,
    }
}

/// Process tracked parameter values and update VarUsageInfo.
pub(crate) fn finalize_param_tracking(
    data: &mut ProgramData,
    fn_id: &Id,
    params: &[Param],
    param_values_map: &FxHashMap<(Id, usize), Vec<Box<Expr>>>,
) {
    for (param_idx, param) in params.iter().enumerate() {
        let Pat::Ident(param_ident) = &param.pat else {
            continue;
        };

        let param_id = param_ident.to_id();
        let key = (fn_id.clone(), param_idx);

        if let Some(values) = param_values_map.get(&key) {
            if values.is_empty() {
                continue;
            }

            // Check if all values are the same
            let first_value = &values[0];
            let all_same = values.iter().all(|v| v.eq_ignore_span(first_value));

            if all_same {
                // Update the parameter's usage info
                if let Some(param_info) = data.vars.get_mut(&param_id) {
                    param_info.param_values = Some(ParameterValues {
                        consistent_value: Some(first_value.clone()),
                        callsite_count: values.len() as u32,
                        is_consistent: true,
                    });
                }
            } else {
                // Mark as inconsistent
                if let Some(param_info) = data.vars.get_mut(&param_id) {
                    param_info.param_values = Some(ParameterValues {
                        consistent_value: None,
                        callsite_count: values.len() as u32,
                        is_consistent: false,
                    });
                }
            }
        }
    }
}