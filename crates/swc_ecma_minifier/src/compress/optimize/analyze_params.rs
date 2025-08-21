use rustc_hash::FxHashMap;
use swc_common::EqIgnoreSpan;
use swc_ecma_ast::*;
use swc_ecma_utils::find_pat_ids;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};

use crate::program_data::{ParameterValues, ProgramData};

/// Analyze parameter values across all callsites to find opportunities for inlining.
pub(crate) fn analyze_param_values(module: &Module, data: &mut ProgramData) {
    let mut analyzer = ParamValueAnalyzer {
        data,
        param_values: FxHashMap::default(),
        function_params: FxHashMap::default(),
    };
    
    // First pass: collect all function parameters
    module.visit_with(&mut analyzer);
    
    // Second pass: finalize the analysis
    analyzer.finalize();
}

struct ParamValueAnalyzer<'a> {
    data: &'a mut ProgramData,
    /// Maps (function_id, param_index) -> list of values passed at callsites
    param_values: FxHashMap<(Id, usize), Vec<Box<Expr>>>,
    /// Maps function_id -> parameter identifiers
    function_params: FxHashMap<Id, Vec<Id>>,
}

impl<'a> ParamValueAnalyzer<'a> {
    /// Check if a value is suitable for tracking (side-effect free and not too large).
    fn is_trackable_value(&self, expr: &Expr) -> bool {
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
    
    /// Track a parameter value for a specific call.
    fn track_param_value(&mut self, fn_id: Id, param_idx: usize, value: &Expr) {
        if !self.is_trackable_value(value) {
            return;
        }
        
        let key = (fn_id, param_idx);
        self.param_values
            .entry(key)
            .or_insert_with(Vec::new)
            .push(Box::new(value.clone()));
    }
    
    /// Finalize the analysis and update VarUsageInfo with parameter value information.
    fn finalize(self) {
        for ((fn_id, param_idx), values) in self.param_values {
            if values.is_empty() {
                continue;
            }
            
            // Get the parameter ID
            let Some(param_ids) = self.function_params.get(&fn_id) else {
                continue;
            };
            
            let Some(param_id) = param_ids.get(param_idx) else {
                continue;
            };
            
            // Check if all values are the same
            let first_value = &values[0];
            let all_same = values.iter().all(|v| v.eq_ignore_span(first_value));
            
            // Update the parameter's usage info
            if let Some(param_info) = self.data.vars.get_mut(param_id) {
                if all_same {
                    param_info.param_values = Some(ParameterValues {
                        consistent_value: Some(first_value.clone()),
                        callsite_count: values.len() as u32,
                        is_consistent: true,
                    });
                } else {
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

impl<'a> Visit for ParamValueAnalyzer<'a> {
    noop_visit_type!();
    
    fn visit_fn_decl(&mut self, n: &FnDecl) {
        // Collect function parameters
        let mut param_ids = vec![];
        for param in &n.function.params {
            if let Pat::Ident(ident) = &param.pat {
                param_ids.push(ident.to_id());
            } else {
                // For non-ident patterns, we can't inline
                param_ids.push(Id::dummy());
            }
        }
        self.function_params.insert(n.ident.to_id(), param_ids);
        
        n.visit_children_with(self);
    }
    
    fn visit_fn_expr(&mut self, n: &FnExpr) {
        // Collect function parameters for named function expressions
        if let Some(ident) = &n.ident {
            let mut param_ids = vec![];
            for param in &n.function.params {
                if let Pat::Ident(param_ident) = &param.pat {
                    param_ids.push(param_ident.to_id());
                } else {
                    param_ids.push(Id::dummy());
                }
            }
            self.function_params.insert(ident.to_id(), param_ids);
        }
        
        n.visit_children_with(self);
    }
    
    fn visit_call_expr(&mut self, n: &CallExpr) {
        n.visit_children_with(self);
        
        // Track parameter values for function calls
        if let Callee::Expr(callee) = &n.callee {
            if let Expr::Ident(fn_ident) = &**callee {
                let fn_id = fn_ident.to_id();
                
                // Check if this is a known function
                if self.function_params.contains_key(&fn_id) {
                    // Track each argument
                    for (param_idx, arg) in n.args.iter().enumerate() {
                        if arg.spread.is_some() {
                            // Stop tracking after spread
                            break;
                        }
                        
                        self.track_param_value(fn_id.clone(), param_idx, &arg.expr);
                    }
                    
                    // Track implicit undefined for missing arguments
                    let param_count = self.function_params.get(&fn_id)
                        .map(|p| p.len())
                        .unwrap_or(0);
                    
                    for param_idx in n.args.len()..param_count {
                        let undefined = Expr::Ident(Ident::new("undefined".into(), Default::default()));
                        self.track_param_value(fn_id.clone(), param_idx, &undefined);
                    }
                }
            }
        }
    }
}