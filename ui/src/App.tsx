import { useState, useEffect } from "react"
import { Routes, Route, useParams, useLocation, useSearchParams } from "react-router-dom"
import { Layout } from "./components/layout/Layout"
import { EntityView } from "./components/views/EntityView"
import { EntityListView } from "./components/views/EntityListView"
import { TagResultsView } from "./components/views/TagResultsView"
import { GraphView } from "./components/views/GraphView"
import { motion, AnimatePresence } from "framer-motion"
import { Share2, Lock, Key, Loader2 } from "lucide-react"
import api, { setToken, getToken } from "./lib/api"

export type ViewMode = "reader" | "raw" | "explorer"

function App() {
  const [viewMode, setViewMode] = useState<ViewMode>("reader")
  const [needsToken, setNeedsToken] = useState(false)
  const location = useLocation()
  const [searchParams, setSearchParams] = useSearchParams()

  // Handle znote token from URL
  useEffect(() => {
    const token = searchParams.get("token")
    if (token) {
      setToken(token)
      setNeedsToken(prev => prev ? false : prev)
      // Remove token from URL to keep it clean
      searchParams.delete("token")
      setSearchParams(searchParams, { replace: true })
    } else if (!getToken()) {
      setNeedsToken(prev => prev ? prev : true)
    }

    const handleUnauthorized = () => setNeedsToken(true)
    window.addEventListener("znote-unauthorized", handleUnauthorized)
    return () => window.removeEventListener("znote-unauthorized", handleUnauthorized)
  }, [searchParams, setSearchParams])

  useEffect(() => {
    const isRootOrList = location.pathname === "/" || /^\/(notes|bookmarks|tasks|tag)\//.test(location.pathname)

    if (isRootOrList && viewMode === "raw") {
      setViewMode(prev => prev === "raw" ? "reader" : prev)
    }
  }, [location.pathname, viewMode])

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "g") {
        e.preventDefault()
        setViewMode(prev => prev === "explorer" ? "reader" : "explorer")
      }
    }
    window.addEventListener("keydown", handleKeyDown)
    return () => window.removeEventListener("keydown", handleKeyDown)
  }, [])

  useEffect(() => {
    api.get("/config").then(res => {
      if (res.data.version) {
        document.title = `znote (v${res.data.version})`
      }
    }).catch(console.error)
  }, [])

  return (
    <>
      <Layout viewMode={viewMode} setViewMode={setViewMode}>
        <AnimatePresence mode="wait">
          {viewMode === "explorer" ? (
            <motion.div
              key="explorer"
              initial={{ opacity: 0, scale: 0.98 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 1.02 }}
              transition={{ duration: 0.4, ease: [0.23, 1, 0.32, 1] }}
            >
              <div className="space-y-6">
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-xl bg-orange-500/10 flex items-center justify-center text-orange-500">
                    <Share2 size={24} />
                  </div>
                  <div>
                    <h2 className="text-xl font-bold tracking-tight leading-none">Knowledge Graph</h2>
                    <p className="text-sm text-muted-foreground mt-1">Spatial visualization of connections.</p>
                  </div>
                </div>
                <GraphView />
              </div>
            </motion.div>
          ) : (
            <motion.div
              key={viewMode}
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 10 }}
              transition={{ duration: 0.3, ease: "easeOut" }}
            >
              <Routes>
                <Route path="/" element={<HomeView />} />
                <Route path="/notes" element={<EntityListView type="note" />} />
                <Route path="/bookmarks" element={<EntityListView type="bookmark" />} />
                <Route path="/tasks" element={<EntityListView type="task" />} />
                <Route path="/tag/*" element={<TagResultsView />} />
                <Route path="/note/:id" element={<DynamicEntityView type="note" viewMode={viewMode} />} />
                <Route path="/bookmark/:id" element={<DynamicEntityView type="bookmark" viewMode={viewMode} />} />
                <Route path="/task/:id" element={<DynamicEntityView type="task" viewMode={viewMode} />} />
              </Routes>
            </motion.div>
          )}
        </AnimatePresence>
      </Layout>

      <TokenModal isOpen={needsToken} onSave={() => setNeedsToken(false)} />
    </>
  )
}

const TokenModal = ({ isOpen, onSave }: { isOpen: boolean; onSave: () => void }) => {
  const [value, setValue] = useState("")

  if (!isOpen) return null

  const handleSave = (e: React.FormEvent) => {
    e.preventDefault()
    if (value.trim()) {
      setToken(value.trim())
      onSave()
    }
  }

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-background/80 backdrop-blur-md">
      <motion.div
        initial={{ opacity: 0, scale: 0.95, y: 10 }}
        animate={{ opacity: 1, scale: 1, y: 0 }}
        className="w-full max-w-sm bg-card border rounded-2xl shadow-2xl p-6 space-y-6"
      >
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center text-primary">
            <Lock size={20} />
          </div>
          <div>
            <h2 className="font-bold text-lg tracking-tight">Authentication Required</h2>
            <p className="text-xs text-muted-foreground">Please enter your ZNote UI token.</p>
          </div>
        </div>

        <form onSubmit={handleSave} className="space-y-4">
          <div className="relative group">
            <Key className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground w-4 h-4" />
            <input
              autoFocus
              type="password"
              placeholder="Enter token..."
              value={value}
              onChange={(e) => setValue(e.target.value)}
              className="w-full bg-muted/40 border-transparent border focus:border-primary focus:bg-background rounded-xl pl-10 pr-4 py-2.5 text-sm focus:outline-none transition-all"
            />
          </div>
          <button
            type="submit"
            className="w-full bg-primary text-primary-foreground font-bold py-2.5 rounded-xl text-sm shadow-lg hover:opacity-90 transition-opacity"
          >
            Access Instance
          </button>
        </form>

        <div className="text-[10px] text-center text-muted-foreground leading-relaxed px-4">
          If you don't have a token, set <code className="bg-muted px-1 rounded">ZNOTE_WEB_UI_TOKEN</code> in your environment before running the server.
        </div>
      </motion.div>
    </div>
  )
}

const HomeView = () => (
  <div className="py-20 text-center space-y-4">
    <div className="w-16 h-16 bg-primary rounded-2xl mx-auto flex items-center justify-center text-primary-foreground font-bold text-3xl italic shadow-xl">z</div>
    <h1 className="text-3xl font-bold tracking-tighter">Welcome to znote</h1>
    <p className="text-muted-foreground max-w-sm mx-auto">Select an entity from the sidebar to find something specific.</p>
  </div>
)

const DynamicEntityView = ({ type, viewMode }: { type: 'note' | 'bookmark' | 'task', viewMode: ViewMode }) => {
  const { id } = useParams()
  const [entity, setEntity] = useState<any>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const fetchData = () => {
      setLoading(true)
      setError(null)
      api.get(`/${type}/${id}`)
        .then(res => {
          setEntity(res.data)
          setLoading(false)
        })
        .catch(err => {
          console.error(err)
          setError(err.response?.data || "Failed to load entity")
          setLoading(false)
        })
    }

    fetchData()
    window.addEventListener("znote-token-changed", fetchData)
    return () => window.removeEventListener("znote-token-changed", fetchData)
  }, [type, id])

  if (loading) {
    return (
      <div className="flex flex-col items-center justify-center py-20 space-y-4">
        <Loader2 className="w-8 h-8 text-primary animate-spin" />
        <p className="text-sm text-muted-foreground italic">Fetching {type}...</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="py-20 text-center space-y-4">
        <div className="w-16 h-16 bg-destructive/10 rounded-2xl mx-auto flex items-center justify-center text-destructive">
          <Lock size={32} />
        </div>
        <h2 className="text-xl font-bold tracking-tight">Error Loading Entity</h2>
        <p className="text-muted-foreground max-w-sm mx-auto">{error}</p>
      </div>
    )
  }

  return (
    <EntityView
      id={id!}
      type={type}
      title={entity.title || (type === 'bookmark' ? entity.url : 'Untitled')}
      tags={entity.tags || []}
      content={type === 'note' ? entity.content : entity.description || ""}
      items={entity.items}
      url={entity.url}
      createdAt={entity.created_at}
      updatedAt={entity.updated_at}
      starred={entity.starred}
      viewMode={viewMode}
    />
  )
}

export default App
