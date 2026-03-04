import React from "react"
import { Sidebar } from "./Sidebar"
import { TopNav } from "./TopNav"
import { CommandPalette } from "./CommandPalette"

import { Terminal, Command, X, Type } from "lucide-react"

import type { ViewMode } from "../../App"

interface LayoutProps {
    children: React.ReactNode
    viewMode: ViewMode
    setViewMode: (mode: ViewMode) => void
}

export const Layout: React.FC<LayoutProps> = ({ children, viewMode, setViewMode }) => {
    return (
        <div className="flex h-screen w-full bg-background overflow-hidden font-sans">
            <Sidebar />
            <div className="flex flex-col flex-1 min-w-0">
                <TopNav viewMode={viewMode} setViewMode={setViewMode} />
                <main className="flex-1 overflow-y-auto p-4 md:p-8">
                    <div className="max-w-4xl mx-auto w-full">
                        {children}
                    </div>
                </main>
            </div>
            <CommandPalette />
            <ShortcutsHelp />
        </div>
    )
}

const ShortcutsHelp: React.FC = () => {
    const [isOpen, setIsOpen] = React.useState(false)

    React.useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "?") {
                setIsOpen(!isOpen)
            }
            if (e.key === "Escape") {
                setIsOpen(false)
            }
        }
        window.addEventListener("keydown", handleKeyDown)
        return () => window.removeEventListener("keydown", handleKeyDown)
    }, [isOpen])

    return (
        <>
            {isOpen && (
                <div
                    className="fixed inset-0 z-[100] flex items-center justify-center bg-background/80 backdrop-blur-md px-4"
                    onClick={() => setIsOpen(false)}
                >
                    <div
                        className="w-full max-w-2xl bg-card border rounded-2xl shadow-2xl overflow-hidden"
                        onClick={e => e.stopPropagation()}
                    >
                        <div className="p-6 border-b flex items-center justify-between bg-muted/30">
                            <div className="flex items-center gap-3">
                                <div className="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center text-primary">
                                    <Command size={20} />
                                </div>
                                <div>
                                    <h2 className="text-xl font-bold tracking-tight">Keyboard Shortcuts</h2>
                                    <p className="text-xs text-muted-foreground">Master znote with these handy commands.</p>
                                </div>
                            </div>
                            <button
                                onClick={() => setIsOpen(false)}
                                className="p-2 hover:bg-muted rounded-full text-muted-foreground"
                            >
                                <X size={20} />
                            </button>
                        </div>

                        <div className="p-8 grid grid-cols-1 md:grid-cols-2 gap-x-12 gap-y-8 h-[400px] overflow-y-auto">
                            <ShortcutSection title="Global">
                                <ShortcutRow keys={["Ctrl", "K"]} label="Open Command Palette" />
                                <ShortcutRow keys={["?"]} label="Show standard shortcuts" />
                                <ShortcutRow keys={["Esc"]} label="Close modal / Deselect" />
                            </ShortcutSection>

                            <ShortcutSection title="Views & Modes">
                                <ShortcutRow keys={["Ctrl", "G"]} label="Toggle Graph Explorer" />
                                <ShortcutRow keys={["Reader"]} label="Reader Mode" icon={<Type size={12} />} />
                                <ShortcutRow keys={["Raw"]} label="Raw Markdown Mode" icon={<Terminal size={12} />} />
                            </ShortcutSection>

                            <ShortcutSection title="Graph Interactions">
                                <ShortcutRow keys={["Click"]} label="Pin / Unpin Node" />
                                <ShortcutRow keys={["Right Click"]} label="Open Entity Detail" />
                                <ShortcutRow keys={["Scroll"]} label="Zoom In / Out" />
                            </ShortcutSection>

                            <ShortcutSection title="Search & Navigation">
                                <ShortcutRow keys={["↑", "↓"]} label="Navigate lists" />
                                <ShortcutRow keys={["Enter"]} label="Select / Open" />
                            </ShortcutSection>
                        </div>

                        <div className="p-4 bg-muted/30 border-t flex items-center justify-center text-[10px] uppercase tracking-[0.2em] font-bold text-muted-foreground/50">
                            znote: knowledge at your fingertips
                        </div>
                    </div>
                </div>
            )}
        </>
    )
}

const ShortcutSection: React.FC<{ title: string; children: React.ReactNode }> = ({ title, children }) => (
    <div className="space-y-4">
        <h3 className="text-[10px] font-bold text-primary uppercase tracking-widest">{title}</h3>
        <div className="space-y-3">{children}</div>
    </div>
)

const ShortcutRow: React.FC<{ keys: string[]; label: string; icon?: React.ReactNode }> = ({ keys, label, icon }) => (
    <div className="flex items-center justify-between group">
        <span className="text-sm text-foreground/80 group-hover:text-foreground flex items-center gap-2">
            {icon}
            {label}
        </span>
        <div className="flex items-center gap-1.5">
            {keys.map(k => (
                <kbd key={k} className="px-1.5 py-0.5 rounded border bg-muted shadow-sm font-mono text-[10px] font-bold min-w-[20px] text-center">
                    {k}
                </kbd>
            ))}
        </div>
    </div>
)
