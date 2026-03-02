import React from "react"
import { Search, Terminal, Sun, Moon, LayoutList, Share2, Monitor, ChevronDown } from "lucide-react"
import { useTheme } from "next-themes"
import { cn } from "@/lib/utils"
import { motion, AnimatePresence } from "framer-motion"

import type { ViewMode } from "../../App"

interface TopNavProps {
    viewMode: ViewMode
    setViewMode: (mode: ViewMode) => void
}

export const TopNav: React.FC<TopNavProps> = ({ viewMode, setViewMode }) => {
    return (
        <header className="h-16 border-b flex items-center justify-between px-6 bg-background sticky top-0 z-10 backdrop-blur-sm bg-background/80">
            <div className="flex-1 flex items-center max-w-lg relative group">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground w-4 h-4 pointer-events-none group-focus-within:text-primary transition-colors" />
                <input
                    type="text"
                    placeholder="Search everything..."
                    className="w-full h-9 bg-muted/40 border-transparent border focus:border-primary focus:bg-background rounded-full pl-10 pr-12 text-sm focus:outline-none transition-all placeholder:text-muted-foreground"
                />
                <div className="absolute right-3 top-1/2 -translate-y-1/2 hidden sm:flex items-center gap-1.5 px-1.5 py-0.5 rounded border bg-background text-[10px] font-bold text-muted-foreground group-focus-within:hidden">
                    <span>Ctrl</span>
                    <span>K</span>
                </div>
            </div>

            <div className="flex items-center gap-4">
                <div className="flex items-center gap-1.5 p-1 border rounded-full bg-muted/20 backdrop-blur-sm shadow-sm scale-90">
                    <ViewToggle active={viewMode === "reader"} icon={<LayoutList size={14} />} label="Reader" onClick={() => setViewMode("reader")} />
                    <ViewToggle active={viewMode === "raw"} icon={<Terminal size={14} />} label="Raw" onClick={() => setViewMode("raw")} />
                    <ViewToggle active={viewMode === "explorer"} icon={<Share2 size={14} />} label="Explorer" onClick={() => setViewMode("explorer")} />
                </div>
                <ThemeToggle />
                <button className="flex items-center gap-2 px-3 py-1.5 rounded-full border bg-background text-xs font-semibold hover:bg-muted transition-colors">
                    <Terminal size={14} className="text-muted-foreground" />
                    <span>Go to CLI</span>
                </button>
            </div>
        </header>
    )
}

const ViewToggle: React.FC<{ active: boolean; icon: React.ReactNode; label: string; onClick: () => void }> = ({ active, icon, label, onClick }) => (
    <button
        onClick={onClick}
        className={`flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[11px] font-semibold transition-all ${active ? "bg-background text-foreground shadow-sm px-4" : "text-muted-foreground hover:text-foreground hover:bg-muted"
            }`}
    >
        {icon}
        <span>{label}</span>
    </button>
)

const ThemeToggle: React.FC = () => {
    const { theme, setTheme } = useTheme()
    const [mounted, setMounted] = React.useState(false)
    const [isOpen, setIsOpen] = React.useState(false)
    const dropdownRef = React.useRef<HTMLDivElement>(null)

    React.useEffect(() => {
        setMounted(true)
        const handleClickOutside = (event: MouseEvent) => {
            if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
                setIsOpen(false)
            }
        }
        document.addEventListener("mousedown", handleClickOutside)
        return () => document.removeEventListener("mousedown", handleClickOutside)
    }, [])

    if (!mounted) return <div className="w-[100px] h-9 border rounded-full bg-muted/20 animate-pulse" />

    const modes = [
        { id: 'light', icon: <Sun size={14} className="text-amber-500" /> },
        { id: 'dark', icon: <Moon size={14} className="text-indigo-400" /> },
        { id: 'system', icon: <Monitor size={14} className="text-muted-foreground" /> }
    ]

    const currentMode = modes.find(m => m.id === theme) || modes[2]

    return (
        <div className="relative" ref={dropdownRef}>
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="flex items-center gap-2 px-3 py-1.5 border rounded-full bg-muted/20 hover:bg-muted/40 transition-all text-xs font-semibold"
            >
                {currentMode.icon}
                <ChevronDown size={12} className={cn("text-muted-foreground transition-transform duration-200", isOpen && "rotate-180")} />
            </button>

            <AnimatePresence>
                {isOpen && (
                    <motion.div
                        initial={{ opacity: 0, y: 8, scale: 0.95 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: 8, scale: 0.95 }}
                        transition={{ duration: 0.15, ease: "easeOut" }}
                        className="absolute right-0 mt-2 p-1.5 w-12 rounded-xl bg-card border shadow-xl z-50 overflow-hidden"
                    >
                        {modes.map((mode) => (
                            <button
                                key={mode.id}
                                onClick={() => {
                                    setTheme(mode.id)
                                    setIsOpen(false)
                                }}
                                className={cn(
                                    "w-full flex items-center justify-center p-2 rounded-lg transition-colors hover:bg-muted mb-0.5 last:mb-0",
                                    theme === mode.id ? "bg-muted text-foreground" : "text-muted-foreground"
                                )}
                            >
                                {mode.icon}
                            </button>
                        ))}
                    </motion.div>
                )}
            </AnimatePresence>
        </div>
    )
}
