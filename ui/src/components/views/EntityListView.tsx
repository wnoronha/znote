import React, { useState, useEffect } from "react"
import { Link } from "react-router-dom"

import {
    ChevronRight,
    Search,
    Loader2,
    Star
} from "lucide-react"
import api, { getToken } from "@/lib/api"
import { TagPill } from "../TagPill"
import { EntityIcon } from "../EntityIcon"
import { TimeToggle } from "../TimeToggle"

interface Entity {
    id: string
    title: string
    tags: string[]
    created_at: string
    updated_at: string
    url?: string
    starred?: boolean
    content?: string
    description?: string
    items?: { text: string; completed: boolean }[]
}

interface EntityListViewProps {
    type: "note" | "bookmark" | "task"
}

export const EntityListView: React.FC<EntityListViewProps> = ({ type }) => {
    const [entities, setEntities] = useState<Entity[]>([])
    const [loading, setLoading] = useState(true)
    const [search, setSearch] = useState("")

    useEffect(() => {
        const fetchData = () => {
            if (!getToken()) {
                setLoading(false)
                return
            }
            setLoading(true)
            api.get(`/${type}s`)
                .then(res => {
                    const sorted = (res.data as Entity[]).sort((a, b) => {
                        if (a.starred && !b.starred) return -1;
                        if (!a.starred && b.starred) return 1;
                        return new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime();
                    })
                    setEntities(sorted)
                    setLoading(false)
                })
                .catch(err => {
                    console.error(err)
                    setLoading(false)
                })
        }

        fetchData()
        window.addEventListener("znote-token-changed", fetchData)
        return () => window.removeEventListener("znote-token-changed", fetchData)
    }, [type])

    const filtered = entities.filter(e =>
        e.title.toLowerCase().includes(search.toLowerCase()) ||
        e.tags.some(t => t.toLowerCase().includes(search.toLowerCase()))
    )

    return (
        <div className="space-y-8">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div className="flex items-center gap-3">
                    <div className="w-12 h-12 rounded-md bg-muted flex items-center justify-center">
                        <EntityIcon type={type} size={20} />
                    </div>
                    <div>
                        <h1 className="text-3xl font-bold tracking-tight capitalize">{type}s</h1>
                        <p className="text-sm text-muted-foreground">{entities.length} total items</p>
                    </div>
                </div>

                <div className="relative w-full md:w-64">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground w-4 h-4" />
                    <input
                        type="text"
                        placeholder={`Search ${type}s...`}
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="w-full h-10 bg-muted/40 border-transparent border focus:border-primary focus:bg-background rounded-xl pl-10 pr-4 text-sm focus:outline-none"
                    />
                </div>
            </div>

            {loading ? (
                <div className="flex flex-col items-center justify-center py-20 space-y-4">
                    <Loader2 className="w-8 h-8 text-primary" />
                    <p className="text-sm text-muted-foreground italic">Loading your {type}s...</p>
                </div>
            ) : filtered.length > 0 ? (
                <div className="grid gap-3">
                    {filtered.map((entity) => (
                        <div key={entity.id}>
                            <Link
                                to={`/${type}/${entity.id}`}
                                className="group block bg-card border hover:border-primary/50 hover:shadow-lg rounded-md p-4 relative overflow-hidden"
                            >
                                <div className="flex items-center justify-between gap-4">
                                    <div className="flex-1 min-w-0 space-y-2">
                                        <div className="flex items-center gap-2">
                                            <h3 className="font-bold text-lg leading-tight truncate group-hover:text-primary">
                                                {entity.title || (type === 'bookmark' ? (entity.url ? new URL(entity.url).hostname : 'Untitled') : 'Untitled')}
                                            </h3>
                                            {entity.starred && <Star size={14} className="text-amber-500 fill-amber-500" />}

                                            {type === 'bookmark' && entity.url && (
                                                <span className="text-[10px] px-2 py-0.5 rounded-md bg-sky-500/10 text-sky-500 font-mono font-bold uppercase tracking-wider border border-sky-500/20">
                                                    {(() => {
                                                        try {
                                                            return new URL(entity.url).hostname.replace('www.', '')
                                                        } catch {
                                                            return 'link'
                                                        }
                                                    })()}
                                                </span>
                                            )}
                                        </div>

                                        {/* Type-Specific Previews */}
                                        {type === 'note' && entity.content && (
                                            <p className="text-sm text-muted-foreground line-clamp-1 italic opacity-70">
                                                {entity.content.replace(/[#*`]/g, '').slice(0, 120)}...
                                            </p>
                                        )}

                                        {type === 'bookmark' && entity.description && (
                                            <p className="text-sm text-muted-foreground line-clamp-1 border-l-2 border-muted pl-2 py-0.5">
                                                {entity.description}
                                            </p>
                                        )}

                                        {type === 'task' && entity.items && entity.items.length > 0 && (
                                            <div className="space-y-1.5 pt-1">
                                                <div className="flex items-center justify-between text-[10px] font-bold uppercase tracking-wider text-muted-foreground">
                                                    <span>
                                                        {entity.items.filter(i => i.completed).length === 0 ? 'Pending' :
                                                            entity.items.filter(i => i.completed).length === entity.items.length ? 'Completed' : 'In Progress'}
                                                    </span>
                                                    <span>{entity.items.filter(i => i.completed).length} / {entity.items.length}</span>
                                                </div>
                                                <div className="h-1.5 w-full bg-muted rounded-full overflow-hidden border border-muted-foreground/5">
                                                    <div
                                                        style={{ width: `${(entity.items.filter(i => i.completed).length / entity.items.length) * 100}%` }}
                                                        className="h-full bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.3)]"
                                                    />
                                                </div>
                                            </div>
                                        )}

                                        <div className="flex flex-wrap items-center gap-2 pt-1">
                                            {entity.tags.map(tag => (
                                                <TagPill key={tag} tag={tag} />
                                            ))}
                                            {entity.tags.length === 0 && <span className="text-[10px] text-muted-foreground italic">no tags</span>}
                                        </div>
                                    </div>

                                    <div className="flex flex-col items-end gap-2 shrink-0 self-start">
                                        <TimeToggle createdAt={entity.created_at} updatedAt={entity.updated_at} minimal />
                                        <ChevronRight size={16} className="text-muted-foreground -translate-x-2 mt-auto" />
                                    </div>
                                </div>
                            </Link>
                        </div>
                    ))}
                </div>
            ) : (
                <div className="py-20 text-center border-2 border-dashed rounded-md space-y-4">
                    <div className="text-muted-foreground italic">No {type}s found matching your search.</div>
                    <button
                        onClick={() => setSearch("")}
                        className="text-sm font-bold text-primary hover:underline"
                    >
                        Clear search
                    </button>
                </div>
            )}
        </div>
    )
}
