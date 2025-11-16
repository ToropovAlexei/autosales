package responses

type LuckGameResponse struct {
	Won        bool    `json:"won"`
	NewBalance float64 `json:"new_balance"`
}
