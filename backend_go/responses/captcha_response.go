package responses

// CaptchaResponse defines the structure for the captcha API response.
type CaptchaResponse struct {
	ImageData string   `json:"image_data" example:"data:image/png;base64,iVBORw0KGgoAAAANSUhEUg..."`
	Answer    string   `json:"answer" example:"A1B2C3"`
	Variants  []string `json:"variants"`
}