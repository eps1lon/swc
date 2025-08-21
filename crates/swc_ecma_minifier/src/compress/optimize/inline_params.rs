use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
use swc_ecma_utils::{ExprExt, Value};

use super::Optimizer;
use crate::{
    compress::optimize::util::is_valid_for_lhs,
    program_data::{ParameterValues, VarUsageInfoFlags},
};

/// Methods related to parameter inlining optimization.
impl Optimizer<'_> {
    /// Track parameter values at callsites for potential inlining.
    pub(super) fn track_param_value_at_callsite(
        &mut self,
        fn_id: &Id,
        param_idx: usize,
        arg: Option<&ExprOrSpread>,
    ) {
        // Get function metadata
        let Some(fn_info) = self.data.vars.get(fn_id) else {
            return;
        };

        // Only track for functions, not other callables
        if !fn_info
            .flags
            .intersects(VarUsageInfoFlags::DECLARED_AS_FN_DECL | VarUsageInfoFlags::DECLARED_AS_FN_EXPR)
        {
            return;
        }

        // For now, we'll need to implement this tracking in the usage analyzer
        // This is a placeholder for the optimization logic
    }

    /// Inline function parameters that are always passed the same value across all callsites.
    #[cfg_attr(feature = "debug", tracing::instrument(level = "debug", skip_all))]
    pub(super) fn inline_params_with_consistent_values(&mut self, f: &mut Function, fn_id: &Id) {
        if !self.options.unused {
            return;
        }

        // Don't inline if function uses eval or with
        if let Some(scope) = self.data.scopes.get(&f.ctxt) {
            if scope.intersects(crate::program_data::ScopeData::HAS_EVAL_CALL | crate::program_data::ScopeData::HAS_WITH_STMT) {
                return;
            }
        }

        // Don't inline if function uses arguments object
        if let Some(scope) = self.data.scopes.get(&f.ctxt) {
            if scope.contains(crate::program_data::ScopeData::USED_ARGUMENTS) {
                return;
            }
        }

        // Check each parameter for consistent values
        let mut params_to_inline = vec![];
        
        for (idx, param) in f.params.iter().enumerate() {
            let Pat::Ident(param_ident) = &param.pat else {
                continue;
            };

            let param_id = param_ident.to_id();
            let Some(param_info) = self.data.vars.get(&param_id) else {
                continue;
            };

            // Check if this parameter has consistent values tracked
            if let Some(ParameterValues {
                consistent_value: Some(value),
                is_consistent: true,
                callsite_count,
            }) = &param_info.param_values
            {
                // Ensure we've seen all callsites
                let Some(fn_info) = self.data.vars.get(fn_id) else {
                    continue;
                };
                
                if fn_info.callee_count != *callsite_count {
                    continue;
                }

                // Check if the value is safe to inline
                if !is_safe_to_inline_value(value, self.ctx.expr_ctx) {
                    continue;
                }

                // Don't inline if parameter is mutated
                if param_info.flags.contains(VarUsageInfoFlags::REASSIGNED) {
                    continue;
                }

                params_to_inline.push((idx, param_id.clone(), value.clone()));
            }
        }

        if params_to_inline.is_empty() {
            return;
        }

        // Apply the inlining
        for (param_idx, param_id, value) in params_to_inline {
            self.changed = true;
            report_change!(
                "inline_params: Inlining parameter '{}' with value '{:?}'",
                param_id.0,
                value
            );

            // Insert a const declaration at the beginning of the function body
            if let Some(body) = &mut f.body {
                let var_decl = VarDecl {
                    span: DUMMY_SP,
                    ctxt: f.ctxt,
                    kind: VarDeclKind::Const,
                    decls: vec![VarDeclarator {
                        span: DUMMY_SP,
                        definite: false,
                        name: Pat::Ident(BindingIdent {
                            id: Ident {
                                span: DUMMY_SP,
                                ctxt: param_id.1,
                                sym: param_id.0.clone(),
                            },
                            type_ann: None,
                        }),
                        init: Some(value),
                    }],
                    ..Default::default()
                };

                body.stmts.insert(0, Stmt::Decl(Decl::Var(Box::new(var_decl))));
            }

            // Mark parameter for removal
            f.params[param_idx].pat = Pat::Invalid(Invalid { span: DUMMY_SP });
        }

        // Remove invalid parameters
        f.params.retain(|p| !p.pat.is_invalid());
    }
}

/// Check if a value is safe to inline.
fn is_safe_to_inline_value(expr: &Expr, expr_ctx: &swc_ecma_utils::ExprCtx) -> bool {
    match expr {
        // Literals are always safe
        Expr::Lit(lit) => {
            match lit {
                // Large strings might cause code bloat
                Lit::Str(s) if s.value.len() > 32 => false,
                _ => true,
            }
        }
        // undefined is safe
        Expr::Ident(i) if &*i.sym == "undefined" => true,
        // Unary operators on literals
        Expr::Unary(UnaryExpr { op: op!("!"), arg, .. }) => {
            matches!(&**arg, Expr::Lit(_))
        }
        Expr::Unary(UnaryExpr { op: op!("-"), arg, .. }) => {
            matches!(&**arg, Expr::Lit(Lit::Num(_)))
        }
        // Other expressions are not safe for now
        _ => false,
    }
}