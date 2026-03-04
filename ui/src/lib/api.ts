import axios from "axios"

const TOKEN_KEY = "znote_web_ui_token"

export const getToken = () => localStorage.getItem(TOKEN_KEY)
export const setToken = (token: string) => {
    console.log("Auth: Setting token", token.slice(0, 4) + "...");
    localStorage.setItem(TOKEN_KEY, token)
    window.dispatchEvent(new Event("znote-token-changed"))
}
export const clearToken = () => {
    console.log("Auth: Clearing token");
    localStorage.removeItem(TOKEN_KEY)
}
const VERSION_KEY = "znote_version"

/**
 * Robustly clear all local storage keys related to znote except the token
 */
export const clearZNoteCache = (clearTokenToo = false) => {
    Object.keys(localStorage).forEach(key => {
        if (key.startsWith("znote_") && key !== TOKEN_KEY) {
            localStorage.removeItem(key)
        }
    })
    if (clearTokenToo) clearToken()
}

/**
 * Check if the server version matches the client version
 */
export const checkVersionSync = (serverVersion: string) => {
    const localVersion = localStorage.getItem(VERSION_KEY)
    if (localVersion && localVersion !== serverVersion) {
        console.warn(`Version mismatch: server=${serverVersion}, local=${localVersion}. Clearing cache.`)
        clearZNoteCache()
    }
    localStorage.setItem(VERSION_KEY, serverVersion)
}

const api = axios.create({
    baseURL: "/api",
})

api.interceptors.request.use((config) => {
    const token = getToken()
    if (token) {
        // Send as headers (more secure than query params as they don't show up in logs/history)
        config.headers["X-ZNote-Token"] = token
        config.headers["Authorization"] = `Bearer ${token}`
    }
    return config
})

// Add response interceptor to handle token expiry/changes
api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            console.error("Auth: Received 401 Unauthorized from server");
            clearToken()
            // Dispatch a custom event to notify the app that authentication failed
            window.dispatchEvent(new Event("znote-unauthorized"))
        }
        return Promise.reject(error)
    }
)

export default api
