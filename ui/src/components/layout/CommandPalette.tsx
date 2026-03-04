import React, { useState, useEffect } from "react"
import { Link } from "react-router-dom"

import { Search, FileText, Bookmark, CheckSquare, Command } from "lucide-react"
import { cn } from "@/lib/utils"

export const CommandPalette: React.FC = () => {
    const [isOpen, setIsOpen] = useState(false)
    const [selectedIndex, setSelectedIndex] = useState(0)

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key === "k") {
                e.preventDefault()
                setIsOpen(!isOpen)
                setSelectedIndex(0)
            }
            if (e.key === "Escape") {
                setIsOpen(false)
            }
            if (isOpen) {
                if (e.key === "ArrowDown") {
                    e.preventDefault()
                    setSelectedIndex(prev => (prev + 1) % 6) // Total items across categories
                }
                if (e.key === "ArrowUp") {
                    e.preventDefault()
                    setSelectedIndex(prev => (prev - 1 + 6) % 6)
                }
            }
        }
        window.addEventListener("keydown", handleKeyDown)
        return () => window.removeEventListener("keydown", handleKeyDown)
    }, [isOpen])

    return (
        <>
            {isOpen && (
                <div
                    className="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-background/80 backdrop-blur-sm"
                    onClick={() => setIsOpen(false)}
                >
                    <div
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
                            <Section title="Navigation">
                                <CommandItem icon={<Search size={16} />} title="Go to Search" onClick={() => setIsOpen(false)} selected={selectedIndex === 4} />
                                <CommandItem icon={<Command size={16} />} title="View Keyboard Shortcuts" onClick={() => { setIsOpen(false); window.dispatchEvent(new KeyboardEvent('keydown', { key: '?' })); }} selected={selectedIndex === 5} />
                            </Section>
                        </div>

                        <div className="p-3 border-t bg-muted/30 flex items-center justify-between text-muted-foreground text-[10px] sm:text-xs">
                            <div className="flex items-center gap-4">
                                <span className="flex items-center gap-1.5"><Kbd>↑↓</Kbd> Navigate</span>
                                <span className="flex items-center gap-1.5"><Kbd>enter</Kbd> Open</span>
                            </div>
                            <span>znote v0.1.0</span>
                        </div>
                    </div>
                </div>
            )}
        </>
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
    onClick: () => void;
    selected?: boolean;
}> = ({ icon, title, id, type, onClick, selected }) => {
    const content = (
        <div
            className={cn(
                "flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer  group",
                selected ? "bg-primary text-primary-foreground shadow-lg scale-[1.02]" : "hover:bg-muted text-foreground"
            )}
            onClick={onClick}
        >
            <div className="flex items-center gap-3">
                <span className={cn("", selected ? "text-primary-foreground" : "text-muted-foreground group-hover:text-primary")}>{icon}</span>
                <span className="text-sm font-medium">{title}</span>
            </div>
            {id && (
                <span className={cn(
                    "text-[10px] font-mono border px-1.5 py-0.5 rounded uppercase",
                    selected ? "bg-white/20 border-white/20 text-white" : "text-muted-foreground bg-muted/50"
                )}>
                    {id.substring(0, 8)}
                </span>
            )}
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
