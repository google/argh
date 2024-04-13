use argh::{FromArgs, TopLevelCommand};

#[derive(FromArgs)]
/// Defines a rectangle
#[argh(verbose_error, help_triggers("-h", "--help"))] 
pub struct Rectangle {
    #[argh(option, short = 'w')]
    /// width; 23 if omitted
    pub width: Option<u32>,

    #[argh(option, short = 'h')]
    /// height; 42 if omitted
    pub height: Option<u32>,

    #[argh(switch)]
    /// print extended help and exit
    pub long_help: bool,

    #[argh(help_text)]
    pub usage: Option<String>,
}

impl Rectangle {
    fn check(&mut self) -> Result<(), String> {
        if self.width.is_none() {
            self.width = Some(23);
        }
        if self.height.is_none() {
            self.height = Some(42);
        }
        let w64: u64 = self.width.unwrap().into();
        let h64: u64 = self.height.unwrap().into();
        let area = w64 * h64;
        if area > 0xFFFFFFFF {
            Err(String::from("You asked for too big a rectangle"))
        } else {
            return Ok(());
        }
    }
}

fn main() {
    let mut rect: Rectangle = argh::from_env();
    if let Err(msg) = rect.check() {
        rect.report_error_and_exit(&msg)
    }
    if rect.long_help {
        println!(
            "{}\n\n{}",
            rect.usage.unwrap(),
            "Definition:
  In Euclidean plane geometry, a rectangle is a quadrilateral with
  four right angles. It can also be defined as: an equiangular
  quadrilateral, since equiangular means that all of its angles are
  equal (360°/4 = 90°); or a parallelogram containing a right angle.
  A rectangle with four sides of equal length is a square. The term
  “oblong” is used to refer to a non-square rectangle.

  According to Wikipedia as of mid April 2024",
        )
    } else {
        let w = rect.width.unwrap();
        let h = rect.height.unwrap();
        println!("Rectangle area is: {}={}x{}", w * h, w, h);
    }
}
