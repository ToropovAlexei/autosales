use captcha::{Captcha, filters::Noise};
use rand::seq::SliceRandom;
use rand::{Rng, distr::Alphanumeric};

use crate::errors::{AppError, AppResult};

pub fn generate_captcha_and_options(
    chars: u32,
    answers: u32,
) -> AppResult<(Vec<u8>, String, Vec<String>)> {
    let mut captcha = Captcha::new();
    captcha.add_chars(chars);
    captcha.apply_filter(Noise::new(0.1));
    let captcha_text = captcha.chars_as_string();
    let captcha_image = captcha.view(360, 90);
    let png_bytes = match captcha_image.as_png() {
        Some(png_bytes) => png_bytes,
        None => {
            return Err(AppError::CaptchaError(
                "Failed to generate captcha".to_string(),
            ));
        }
    };

    let mut options = vec![captcha_text.clone()];
    let mut rng = rand::rng();

    while options.len() < answers as usize {
        let option: String = (0..chars)
            .map(|_| {
                let c = rng.sample(Alphanumeric) as char;
                c.to_ascii_uppercase()
            })
            .collect();

        if !options.contains(&option) {
            options.push(option);
        }
    }

    options.shuffle(&mut rng);

    Ok((png_bytes, captcha_text, options))
}
