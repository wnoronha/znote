import React, { useState, useEffect } from "react"
import { cn } from "@/lib/utils"
import { Link, useLocation } from "react-router-dom"
import {
    Settings,
    MoreVertical,
    ChevronDown,
    Loader2,
    Star,
    Fan,
    HelpCircle
} from "lucide-react"

import api, { getToken } from "@/lib/api"
import { EntityIcon } from "../EntityIcon"

export const Sidebar: React.FC = () => {
    const location = useLocation()
    const [notes, setNotes] = useState<any[]>([])
    const [bookmarks, setBookmarks] = useState<any[]>([])
    const [tasks, setTasks] = useState<any[]>([])
    const [tags, setTags] = useState<string[]>([])
    const [starredTag, setStarredTag] = useState<string>("#starred")
    const [version, setVersion] = useState<string>("")
    const [loading, setLoading] = useState(true)


    useEffect(() => {
        const fetchData = async (silent = false) => {
            if (!getToken()) {
                setLoading(false)

                return
            }
            if (!silent) setLoading(true)

            try {
                const [n, b, t, tg, cfg] = await Promise.all([
                    api.get("/notes"),
                    api.get("/bookmarks"),
                    api.get("/tasks"),
                    api.get("/tags"),
                    api.get("/config")
                ])
                setNotes(n.data)
                setBookmarks(b.data)
                setTasks(t.data)
                setTags(tg.data)
                setStarredTag(cfg.data.starred_tag)
                setVersion(cfg.data.version)
            } catch (err) {
                console.error("Failed to fetch sidebar data", err)
            } finally {
                setLoading(false)
            }
        }

        fetchData()
        const interval = setInterval(() => fetchData(true), 30000) // Poll silently every 30s

        const onTokenChanged = () => fetchData()
        window.addEventListener("znote-token-changed", onTokenChanged)
        return () => {
            clearInterval(interval)
            window.removeEventListener("znote-token-changed", onTokenChanged)
        }
    }, [])

    const filterStarred = (items: any[]) => items.filter(item => item.tags?.includes(starredTag))
    const starredNotes = filterStarred(notes)
    const starredBookmarks = filterStarred(bookmarks)
    const starredTasks = filterStarred(tasks)

    return (
        <aside className="w-64 h-full border-r bg-muted/30 flex flex-col hidden md:flex shrink-0">
            <Link to="/" className="p-4 flex items-center gap-2 border-b hover:bg-muted/50">
                <div className="flex items-center gap-2">
                    <div
                        className="w-6 h-6 bg-primary rounded-md flex items-center justify-center text-primary-foreground shadow-sm shrink-0"
                    >
                        <Fan size={14} strokeWidth={2.5} />
                    </div>

                    <span className="font-bold text-sm tracking-tight text-center">
                        znote
                    </span>
                </div>
            </Link>

            <nav className="flex-1 overflow-y-auto p-3 space-y-4">
                <div className="space-y-1">
                    <SidebarItem
                        icon={<EntityIcon type="note" size={16} />}
                        label="Notes"
                        href="/notes"
                        isActive={location.pathname === "/notes"}
                    />
                    <SidebarItem
                        icon={<EntityIcon type="bookmark" size={16} />}
                        label="Bookmarks"
                        href="/bookmarks"
                        isActive={location.pathname === "/bookmarks"}
                    />
                    <SidebarItem
                        icon={<EntityIcon type="task" size={16} />}
                        label="Tasks"
                        href="/tasks"
                        isActive={location.pathname === "/tasks"}
                    />
                </div>

                {loading ? (
                    <div className="flex items-center justify-center py-4">
                        <Loader2 size={16} className="text-muted-foreground" />
                    </div>
                ) : (
                    <>
                        {starredNotes.length > 0 && <SidebarSection title="Notes" icon={<EntityIcon type="note" size={16} />} items={starredNotes} type="note" currentPath={location.pathname} />}
                        {starredBookmarks.length > 0 && <SidebarSection title="Bookmarks" icon={<EntityIcon type="bookmark" size={16} />} items={starredBookmarks} type="bookmark" currentPath={location.pathname} />}
                        {starredTasks.length > 0 && <SidebarSection title="Tasks" icon={<EntityIcon type="task" size={16} />} items={starredTasks} type="task" currentPath={location.pathname} />}
                    </>
                )}

                <div>
                    <div className="px-3 mb-2 flex items-center justify-between group">
                        <span className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest">Tags</span>
                        <MoreVertical size={12} className="text-muted-foreground cursor-pointer" />
                    </div>
                    <div className="space-y-0.5 px-3">
                        {tags.map(tag => {
                            const cleanTag = tag.startsWith('#') ? tag.slice(1) : tag;
                            const isActive = location.pathname === `/tag/${cleanTag}`;
                            return (
                                <Link
                                    key={tag}
                                    to={`/tag/${cleanTag}`}
                                    className={cn(
                                        "text-xs font-mono block truncate py-1 hover:underline ",
                                        isActive ? "text-primary font-bold" : "text-muted-foreground"
                                    )}
                                >
                                    {tag.startsWith('#') ? tag : `#${tag}`}
                                </Link>
                            );
                        })}
                        {!loading && tags.length === 0 && (
                            <span className="text-[10px] text-muted-foreground italic px-1">No tags found</span>
                        )}
                    </div>
                </div>
            </nav>

            <div className="p-4 border-t space-y-2">
                <SidebarItem
                    icon={<HelpCircle size={16} />}
                    label="Shortcuts"
                    onClick={() => window.dispatchEvent(new KeyboardEvent('keydown', { key: '?' }))}
                />
                <SidebarItem icon={<Settings size={16} />} label="Settings" />
                <div className="px-3 py-1 text-[10px] text-muted-foreground font-mono opacity-50">
                    znote v{version}
                </div>
            </div>
        </aside>
    )
}

const SidebarSection: React.FC<{
    title: string;
    icon: React.ReactNode;
    items: any[];
    type: string;
    currentPath: string;
}> = ({ title, icon, items, type, currentPath }) => {
    const [isOpen, setIsOpen] = useState(true)

    return (
        <div>
            <div className="flex items-center justify-between px-3 mb-1">
                <Link
                    to={`/${type}s`}
                    className="text-[10px] font-bold text-muted-foreground uppercase tracking-widest hover:text-primary"
                >
                    {title}
                </Link>
                <button
                    onClick={() => setIsOpen(!isOpen)}
                    className="text-muted-foreground hover:text-foreground"
                >
                    <ChevronDown size={12} className={cn("", !isOpen && "-rotate-90")} />
                </button>
            </div>
            {isOpen && (
                <div className="space-y-1">
                    {items.slice(0, 5).map(item => (
                        <SidebarItem
                            key={item.id}
                            icon={item.starred ? <Star size={12} className="text-amber-500 fill-amber-500" /> : icon}
                            label={item.title}
                            href={`/${type}/${item.id}`}
                            isActive={currentPath === `/${type}/${item.id}`}
                        />
                    ))}
                    {items.length > 5 && (
                        <Link
                            to={`/${type}s`}
                            className="block px-9 py-1 text-[10px] text-muted-foreground italic hover:text-primary"
                        >
                            + {items.length - 5} more
                        </Link>
                    )}
                </div>
            )}
        </div>
    )
}

const SidebarItem: React.FC<{
    icon: React.ReactNode;
    label: string;
    isActive?: boolean;
    href?: string;
    onClick?: () => void;
}> = ({ icon, label, isActive, href, onClick }) => {
    const content = (
        <div
            onClick={onClick}
            className={cn(
                "flex items-center gap-3 px-3 py-1.5 rounded-md cursor-pointer text-sm font-medium ",
                isActive ? "bg-accent text-accent-foreground" : "hover:bg-muted text-muted-foreground hover:text-foreground"
            )}
        >
            <span className="opacity-70 shrink-0">{icon}</span>
            <span className="truncate">{label}</span>
        </div>
    )

    if (href) {
        return <Link to={href}>{content}</Link>
    }

    return content
}
