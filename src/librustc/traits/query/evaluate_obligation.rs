// Copyright 2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use infer::InferCtxt;
use smallvec::SmallVec;
use traits::{EvaluationResult, PredicateObligation, SelectionContext,
             TraitQueryMode, OverflowError};

impl<'cx, 'gcx, 'tcx> InferCtxt<'cx, 'gcx, 'tcx> {
    /// Evaluates whether the predicate can be satisfied (by any means)
    /// in the given `ParamEnv`.
    pub fn predicate_may_hold(
        &self,
        obligation: &PredicateObligation<'tcx>,
    ) -> bool {
        self.evaluate_obligation_no_overflow(obligation).may_apply()
    }

    /// Evaluates whether the predicate can be satisfied in the given
    /// `ParamEnv`, and returns `false` if not certain. However, this is
    /// not entirely accurate if inference variables are involved.
    pub fn predicate_must_hold(
        &self,
        obligation: &PredicateObligation<'tcx>,
    ) -> bool {
        self.evaluate_obligation_no_overflow(obligation) == EvaluationResult::EvaluatedToOk
    }

    /// Evaluate a given predicate, capturing overflow and propagating it back.
    pub fn evaluate_obligation(
        &self,
        obligation: &PredicateObligation<'tcx>,
    ) -> Result<EvaluationResult, OverflowError> {
        let mut _orig_values = SmallVec::new();
        let c_pred = self.canonicalize_query(&obligation.param_env.and(obligation.predicate),
                                             &mut _orig_values);
        // Run canonical query. If overflow occurs, rerun from scratch but this time
        // in standard trait query mode so that overflow is handled appropriately
        // within `SelectionContext`.
        self.tcx.global_tcx().evaluate_obligation(c_pred)
    }

    // Helper function that canonicalizes and runs the query. If an
    // overflow results, we re-run it in the local context so we can
    // report a nice error.
    fn evaluate_obligation_no_overflow(
        &self,
        obligation: &PredicateObligation<'tcx>,
    ) -> EvaluationResult {
        match self.evaluate_obligation(obligation) {
            Ok(result) => result,
            Err(OverflowError) => {
                let mut selcx =
                    SelectionContext::with_query_mode(&self, TraitQueryMode::Standard);
                selcx.evaluate_obligation_recursively(obligation)
                    .unwrap_or_else(|r| {
                        span_bug!(
                            obligation.cause.span,
                            "Overflow should be caught earlier in standard query mode: {:?}, {:?}",
                            obligation,
                            r,
                        )
                    })
            }
        }
    }
}
