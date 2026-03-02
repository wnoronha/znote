import axios from "axios"

const TOKEN_KEY = "znote_web_ui_token"

export const getToken = () => localStorage.getItem(TOKEN_KEY)
export const setToken = (token: string) => {
    localStorage.setItem(TOKEN_KEY, token)
    window.dispatchEvent(new Event("znote-token-changed"))
}
export const clearToken = () => localStorage.removeItem(TOKEN_KEY)

const api = axios.create({
    baseURL: "/api",
})

api.interceptors.request.use((config) => {
    const token = getToken()
    if (token) {
        // Send as headers
        config.headers["X-ZNote-Token"] = token
        config.headers["Authorization"] = `Bearer ${token}`
        
        // Send as query param
        config.params = {
            ...config.params,
            token: token,
        }
    }
    return config
})

// Add response interceptor to handle token expiry/changes
api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            clearToken()
            // Dispatch a custom event to notify the app that authentication failed
            window.dispatchEvent(new Event("znote-unauthorized"))
        }
        return Promise.reject(error)
    }
)

export default api
