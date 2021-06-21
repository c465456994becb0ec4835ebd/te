mod account;
mod transaction;

use std::env;

use csv::{ReaderBuilder, Trim};

use transaction::TransactionEngine;

fn main() {
    let input_path = env::args()
        .nth(1)
        .expect("Please specify the input file path.");

    let mut engine = TransactionEngine::new();

    // Seems like `csv::Reader` already performs some internal buffering. If that's not
    // sufficient, we could open the input file ourselves and use/implement some other
    // sort of buffering logic.
    let mut reader = ReaderBuilder::new()
        // Required for the csv logic to ignore whitespaces; otherwise, the presence of
        // any whitespace seems to cause errors.
        .trim(Trim::All)
        // Setting this so we can have rows where the amount is not explicitly specified
        // (i.e. dispute-related transactions)
        .flexible(true)
        .from_path(input_path)
        .expect("Unable to open the input file");

    // This loop incrementally processes the input data, and attempts to deserialize
    // one record at a time.
    for result in reader.deserialize() {
        if let Ok(t) = result {
            // We could examine the result below to perform additional logic for the different
            // reasons why a transaction was not committed successfully (i.e. insufficient
            // funds). We simply move to the next transaction for now.
            let _process_result = engine.process_transaction(t);
        } else {
            // If we got here, then parsing one of the rows has failed. Let's just ignore
            // invalid records for this simple program, and continue.
        }
    }

    engine.print_accounts()
}
