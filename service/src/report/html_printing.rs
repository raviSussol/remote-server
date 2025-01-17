use std::fs;

use headless_chrome::{protocol::page::PrintToPdfOptions, Browser, LaunchOptionsBuilder};

use super::report_service::GeneratedReport;

pub fn html_to_pdf(
    document: &GeneratedReport,
    document_id: &str,
) -> Result<Vec<u8>, failure::Error> {
    let pdf_options = Some(PrintToPdfOptions {
        display_header_footer: Some(true),
        prefer_css_page_size: Some(false),
        landscape: Some(true),
        print_background: None,
        scale: None,
        // Assuming 96 DPI (dots per inch)
        paper_width: Some(8.0),
        paper_height: Some(11.0),
        margin_top: Some(1.0),
        margin_bottom: Some(1.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        header_template: document.header.clone(),
        footer_template: document.footer.clone(),
    });

    let document_name = format!("{}.html", document_id);
    let temp_html_doc_path = std::env::current_dir()?.join(document_name);
    //let file_path = PathBuf::from_str(&document_name)?.canonicalize()?;
    fs::write(&temp_html_doc_path, &document.document)?;

    // create a new browser and a tab in that browser using headless-chrome
    let launch_options = LaunchOptionsBuilder::default()
        .headless(true)
        .build()
        .map_err(|err| failure::err_msg(err))?;
    let browser = Browser::new(launch_options)?;
    let tab = browser.wait_for_initial_tab()?;
    let local_pdf = tab
        .navigate_to(&format!("file:{}", temp_html_doc_path.to_string_lossy()))?
        .wait_until_navigated()?
        .print_to_pdf(pdf_options)?;

    // clean up
    fs::remove_file(temp_html_doc_path)?;
    Ok(local_pdf)
}
