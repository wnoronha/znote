import React, { useState, useEffect } from "react"
import { Link } from "react-router-dom"
import { motion, AnimatePresence } from "framer-motion"
import { Search, FileText, Bookmark, CheckSquare, Settings } from "lucide-react"

export const CommandPalette: React.FC = () => {
    const [isOpen, setIsOpen] = useState(false)

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key === "k") {
                e.preventDefault()
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
        <AnimatePresence>
            {isOpen && (
                <div
                    className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-background/80 backdrop-blur-sm"
                    onClick={() => setIsOpen(false)}
                >
                    <motion.div
                        initial={{ scale: 0.95, opacity: 0 }}
                        animate={{ scale: 1, opacity: 1 }}
                        exit={{ scale: 0.95, opacity: 0 }}
                        className="w-full max-w-xl bg-card border rounded-xl shadow-2xl overflow-hidden"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <div className="p-4 border-b flex items-center gap-3">
                            <Search className="text-muted-foreground w-4 h-4" />
                            <input
                                autoFocus
                                type="text"
                                placeholder="What do you need?"
                                className="w-full bg-transparent border-none outline-none text-sm placeholder:text-muted-foreground"
                            />
                            <span className="text-[10px] font-bold text-muted-foreground border px-1.5 py-0.5 rounded uppercase">esc</span>
                        </div>

                        <div className="p-2 overflow-y-auto max-h-[360px] space-y-4">
                            <Section title="Notes">
                                <CommandItem icon={<FileText size={16} />} title="Understanding Rust Ownership" id="886a5bea-1234-4567-8901-23456789abcd" type="note" onClick={() => setIsOpen(false)} />
                                <CommandItem icon={<FileText size={16} />} title="Project: znote Core" id="e5f6g7h8-9012-3456-7890-abcdef123456" type="note" onClick={() => setIsOpen(false)} />
                            </Section>
                            <Section title="Bookmarks">
                                <CommandItem icon={<Bookmark size={16} />} title="Rust Docs" id="ba890342-47b6-4fdc-b0f5-6e4311ab30de" type="bookmark" onClick={() => setIsOpen(false)} />
                            </Section>
                            <Section title="Tasks">
                                <CommandItem icon={<CheckSquare size={16} />} title="Ship v1" id="e5f6g7h8-9012-3456-7890-abcdef123456" type="task" onClick={() => setIsOpen(false)} />
                            </Section>
                            <Section title="Settings">
                                <CommandItem icon={<Settings size={16} />} title="Settings & Config" onClick={() => setIsOpen(false)} />
                            </Section>
                        </div>

                        <div className="p-3 border-t bg-muted/30 flex items-center justify-between text-muted-foreground text-[10px] sm:text-xs">
                            <div className="flex items-center gap-4">
                                <span className="flex items-center gap-1.5"><Kbd>↑↓</Kbd> Navigate</span>
                                <span className="flex items-center gap-1.5"><Kbd>enter</Kbd> Open</span>
                            </div>
                            <span>znote v0.1.0</span>
                        </div>
                    </motion.div>
                </div>
            )}
        </AnimatePresence>
    )
}

const Section: React.FC<{ title: string; children: React.ReactNode }> = ({ title, children }) => (
    <div className="space-y-1">
        <div className="px-3 text-[10px] font-bold text-muted-foreground uppercase tracking-widest py-1">{title}</div>
        {children}
    </div>
)

const CommandItem: React.FC<{
    icon: React.ReactNode;
    title: string;
    id?: string;
    type?: 'note' | 'bookmark' | 'task';
    onClick: () => void
}> = ({ icon, title, id, type, onClick }) => {
    const content = (
        <div className="flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer transition-colors hover:bg-muted group" onClick={onClick}>
            <div className="flex items-center gap-3">
                <span className="text-muted-foreground group-hover:text-primary transition-colors">{icon}</span>
                <span className="text-sm font-medium">{title}</span>
            </div>
            {id && <span className="text-[10px] font-mono text-muted-foreground border px-1.5 py-0.5 rounded bg-muted/50 uppercase">{id.substring(0, 8)}</span>}
        </div>
    )

    if (id && type) {
        return <Link to={`/${type}/${id}`}>{content}</Link>
    }

    return content
}

const Kbd: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <span className="text-[10px] font-bold border rounded px-1.5 py-0.5 min-w-[1.25rem] bg-background text-muted-foreground shadow-sm h-5 inline-flex items-center">
        {children}
    </span>
)
