//! Interfaces between the pipeline (consumer) and adapters (providers).
//!
//! Rules (see SPEC.md, "Interface rules"):
//! - signatures use only `flatart-domain` types;
//! - each trait documents its contract;
//! - each trait gets a conformance suite in `testkit` that every adapter runs.
//!
//! Traits are added by work-split task 03 (domain-and-ports).
