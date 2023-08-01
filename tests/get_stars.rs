use std::fs::File;

use elv::{self, domain::stars::Stars};
mod common;

struct ResponseMock {
    file: File,
}

impl std::io::Read for ResponseMock {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

#[test]
fn test_printing_year_2022() {
    let mock = ResponseMock {
        file: common::get_resource_as_file("stars-page-full-stars.html"),
    };
    let stars: Stars = Stars::from_readable(mock).unwrap();
    let expected = r#"
  - /\ -  -        -       -     -      -    -
 - /  \/\  -    -     -  -    -   -  /\   -     -
 /\    \ \-  - -   -   -    - -  -/\/  \-   -  -   25 **
/@@\   /\@\@@@@@@@@@#@@#@#@@@@@@@#@@@##@@@@@#@#@#  24 **
@##.' '.#@@@#@@@@@@#@@#@@@##@#@@@##@##@@@@@@@@@@@  23 **
@#@'. .'@#@@.@@##@@@@@#####@@@@@##@@@@@#@()))@@@#  22 **
#@#@@@@####@@@#@##.@@@@@#@@#@@@##@@@@@@@#@@@@@@#@  21 **
#@@@@.@@#@#@@@@###@#@#@@.#@@~~@@@@@#@@@@#@@@@@@@#  20 **
#@@@@@##@#@@#.#@#@@@@##@#@@~~~~ .~'@@@##@@#@#@@@@  19 **
@#####@#@@@#@@.@@#@@@@@#@##@~~ /~\ ##@@#@####@#@|  18 **
##@@.@@#@@@#@@@..#@@@@@@@@@@@ / / \ #@@@@@##@#@@@  17 **
@@#@##@@#@@@@##@..@@##@@@@@#@/ / \ \@@@@@@@@@@###  16 **
@#@@#@@@@@@#@@#_.~._#@#@#@#@.'/\.'~. @#@@@@@#@@#@  15 **
#@#.@@@@@@@@@@@ ||| @#@@#@@'.~.'~. \'.@@@@@@@##@@  14 **
@@@@@@@@@@@#@@#@~~~@#@@@##@@' ..'.'.\. . #@@###@@  13 **
@@@@@@@#@@@@@#.~~.#@@#@#@@@@@@@@@ .'.~~~' @'@##@@  12 **
@@@.@#@#@@@@.~~.###@#@@@@@@@@#@@@@  ~~~~~..#@@@##  11 **
#@@.@##@@@#.~~.#@@@@@#@#@@@@@@@#@.'/ ~~~ \' @#@@#  10 **
#@@@.@@ _|%%%=%%|_ #@@@#@@#@@@@@. ~ /' .'/\.@@#@@   9 **
#@#@@../  \.~~./  \.....@@#@@###@' /\.''/' \' @#@   8 **
@@#@#@#@@@@.~~.@#@@@@@@#.@@##@#@'././\ .\'./\ '.    7 **
@@@@@@##@#@@.~~.@##@@#@..@@@##@' ~. \.\  \ \.~~~.   6 **
@@@#@@@@@@###.~~.##./\.'@@@@@@@@'.' .'/.@. /'.~~~   5 **
@@@##@@#@@#.' ~  './\'./\' .@@@@@#@' /\ . /'. ..'   4 **
@#@@#@@@#_/ ~   ~  \ ' '. '.'.@@@@  /  \  \  #@@@   3 **
-~------'    ~    ~ '--~-----~-~----___________--   2 **
  ~    ~  ~      ~     ~ ~   ~     ~  ~  ~   ~      1 **
"#
    .to_owned();
    std::iter::zip(expected.split("\n"), format!("\n{}", stars).split("\n")).for_each(
        |(expected, result)| {
            assert_eq!(expected, result.trim_end());
        },
    )
}
