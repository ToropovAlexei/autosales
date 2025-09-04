import random
import string
from io import BytesIO
from captcha.image import ImageCaptcha

def generate_captcha_and_options():
    image = ImageCaptcha(width=360, height=90)
    captcha_text = ''.join(random.choices(string.ascii_uppercase + string.digits, k=6))
    data = image.generate(captcha_text)
    
    options = [captcha_text]
    while len(options) < 12:
        random_text = ''.join(random.choices(string.ascii_uppercase + string.digits, k=6))
        if random_text not in options:
            options.append(random_text)
    
    random.shuffle(options)
    
    return data, captcha_text, options