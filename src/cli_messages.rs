use crate::types::ProcessingSummary;

pub fn print_console_summary(summary: &ProcessingSummary, verbose: bool) {
    let success_count = summary.get_success_count();
    let failed_count = summary.get_failed_count();
    if verbose {
        println!("\n=== PROCESSING SUMMARY ===\n");

        for result in &summary.results {
            let status_icon = if result.success { "✓" } else { "✗" };
            println!("{} File: {}", status_icon, result.file_path);

            if let Some(error) = &result.error_message {
                println!("  Error: {error}");
                continue;
            }

            if result.includes.is_empty() {
                println!("  No includes found");
            } else {
                println!("  Includes ({}):", result.includes.len());
                for include in &result.includes {
                    let include_icon = if include.success { "✓" } else { "✗" };
                    let status_text = if include.success { "OK" } else { "Error" };
                    println!("    {} {}: {}", include_icon, status_text, include.path);

                    if let Some(error) = &include.error_message {
                        println!("      └─ {error}");
                    }
                }
            }
            println!();
        }

        // Final statistics
        println!("=== FINAL SUMMARY ===");
        println!("┌─────────────────┬───────┬─────────┬────────┐");
        println!("│ Category        │ Total │ Success │ Failed │");
        println!("├─────────────────┼───────┼─────────┼────────┤");
        println!(
            "│ Files           │ {:>5} │ {:>7} │ {:>6} │",
            summary.results.len(),
            success_count,
            failed_count
        );
        println!(
            "│ Includes        │ {:>5} │ {:>7} │ {:>6} │",
            summary.get_total_includes(),
            summary.get_successful_includes(),
            summary.get_failed_includes()
        );
        println!("└─────────────────┴───────┴─────────┴────────┘");

        if summary.get_failed_count() > 0 || summary.get_failed_includes() > 0 {
            println!("\nSome operations failed. Check the details above.");
        } else {
            println!("\nAll operations completed successfully! 🎉");
        }
    } else {
        // Simple summary for non-verbose mode
        let success_count = success_count;
        let failed_count = failed_count;
        println!(
            "Processed {} files: {} succeeded, {} failed.",
            summary.results.len(),
            success_count,
            failed_count
        );
        if failed_count > 0 {
            println!("Some files failed to process.");
            std::process::exit(1);
        } else {
            println!("All files processed successfully!");
            std::process::exit(0);
        }
    }
}
