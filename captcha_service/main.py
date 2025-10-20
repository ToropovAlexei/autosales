from flask import Flask, jsonify
from captcha_helper import generate_captcha
import base64

app = Flask(__name__)

@app.route('/captcha', methods=['GET'])
def get_captcha():
    image_bytes, solution, options = generate_captcha()
    image_base64 = base64.b64encode(image_bytes.getvalue()).decode('utf-8')
    return jsonify({
        'image': image_base64,
        'solution': solution
    })

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5001)
