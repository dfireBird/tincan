use std::error::Error;

use tincan::ui;
fn main() -> Result<(), Box<dyn Error>> {
    ui::start_ui()
}
