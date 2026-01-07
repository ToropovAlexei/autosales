package main

import (
	"encoding/json"
	"log"
	"math/rand"
	"net/http"
	"time"

	"github.com/mojocn/base64Captcha"
)

type CaptchaResponse struct {
	Answer    string   `json:"answer"`
	Variants  []string `json:"variants"`
	ImageData string   `json:"img"`
}

func GetCaptchaHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")

	height := 80
	width := 240
	length := 6
	variantsLength := 12

	source := "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
	driver := base64Captcha.NewDriverString(height, width, 0, base64Captcha.OptionShowHollowLine, length, source, nil, nil, nil)
	captcha := base64Captcha.NewCaptcha(driver, base64Captcha.DefaultMemStore)

	_, content, answer, err := captcha.Generate()
	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(map[string]string{"error": "Failed to generate captcha"})
		return
	}

	if content == "" {
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(map[string]string{"error": "Failed to generate captcha image"})
		return
	}

	variantsMap := make(map[string]struct{})
	variantsMap[answer] = struct{}{}
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))

	for len(variantsMap) < variantsLength {
		b := make([]byte, length)
		for i := range b {
			b[i] = source[rng.Intn(len(source))]
		}
		variantsMap[string(b)] = struct{}{}
	}

	variants := make([]string, 0, len(variantsMap))
	for v := range variantsMap {
		variants = append(variants, v)
	}

	rng.Shuffle(len(variants), func(i, j int) { variants[i], variants[j] = variants[j], variants[i] })

	response := CaptchaResponse{
		ImageData: content,
		Answer:    answer,
		Variants:  variants,
	}

	w.WriteHeader(http.StatusOK)
	if err := json.NewEncoder(w).Encode(response); err != nil {
		log.Printf("Error encoding response: %v", err)
	}
}

func main() {
	http.HandleFunc("/captcha", GetCaptchaHandler)
	log.Println("Captcha service starting on :9091")
	if err := http.ListenAndServe(":9091", nil); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}
