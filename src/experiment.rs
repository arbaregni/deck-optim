/// The outcome of a single experiment
struct ExperimentResult {
    param: usize,
    measure: f32
}

/*
 *let results = (0..TOTAL_DECK_SIZE)
        .into_iter()
        .map(|param| {
            let metrics = evaluate_deck(&cli, num_trials, param);
            let mut measure = metrics.average("turn-to-reach-7-plays");
            if measure == 0.0 {
                measure = 9999.0;
            }
            ExperimentResult {
                param,
                measure
            }
        })
        .collect_vec();

    // select the best parameter
    let best = results
        .iter()
        .min_by(|exp1, exp2| exp1.measure.partial_cmp(&exp2.measure).expect("no nan"));

    let mut table = make_table();
    table.set_titles(row!["# of lightning bolts", "turns until 21 damage", ""]);
    for exp in results.iter() {
        let is_best = match best {
            Some(best_exp) if best_exp.param == exp.param => "BEST",
            _ => ""
        };
        table.add_row(row![exp.param, exp.measure, is_best]);
    }
    println!();
    println!();
    println!("===================================================");
    table.printstd();

*/


