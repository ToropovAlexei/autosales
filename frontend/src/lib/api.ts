import ky from "ky";
import { CONFIG } from "../../config";

const getAuthToken = () => {
  if (typeof window !== "undefined") {
    return localStorage.getItem("jwt");
  }
  return null;
};

export const newApi = ky.extend({
  prefixUrl: CONFIG.API_URL,
  timeout: 30000,
  hooks: {
    beforeRequest: [
      (request) => {
        const token = getAuthToken();
        if (token) {
          request.headers.set("Authorization", `Bearer ${token}`);
        }
      },
    ],
    afterResponse: [
      (request, options, response) => {
        if (response.status === 401 || response.status === 403) {
          localStorage.removeItem("jwt");
          if (typeof window !== "undefined") {
            window.location.href = "/login";
          }
        }
        return response;
      },
    ],
  },
});
