import React, { useState, useEffect } from "react"
import { Link, useLocation } from "react-router-dom"
import { motion } from "framer-motion"
import {
    ChevronRight,
    Loader2,
    Hash,
    Lock,
    Star
} from "lucide-react"
import api from "@/lib/api"
import { TagPill } from "../TagPill"
import { EntityIcon } from "../EntityIcon"
import { TimeToggle } from "../TimeToggle"

interface UnifiedEntity {
    id: string
    title: string
    tags: string[]
    created_at: string
    updated_at: string
    type: "note" | "bookmark" | "task"
    url?: string
    starred?: boolean
}

export const TagResultsView: React.FC = () => {
    const location = useLocation()
    const tag = decodeURIComponent(location.pathname.replace(/^\/tag\//, ''))
    const [entities, setEntities] = useState<UnifiedEntity[]>([])
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState<string | null>(null)

    useEffect(() => {
        const fetchData = () => {
            setLoading(true)
            setError(null)
            // Use the existing query subcommand logic: tag:value
            api.get(`/query?expr=tag:${tag}`)
                .then(res => {
                    setEntities(res.data)
                    setLoading(false)
                })
                .catch(err => {
                    console.error(err)
                    setError(err.response?.data || "Failed to load results")
                    setLoading(false)
                })
        }

        fetchData()
        window.addEventListener("znote-token-changed", fetchData)
        return () => window.removeEventListener("znote-token-changed", fetchData)
    }, [tag])

    return (
        <div className="space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div className="flex items-center gap-3">
                    <div className="w-12 h-12 rounded-2xl bg-primary/10 flex items-center justify-center text-primary">
                        <Hash size={24} />
                    </div>
                    <div>
                        <h1 className="text-3xl font-bold tracking-tight">#{tag}</h1>
                        <p className="text-sm text-muted-foreground">{entities.length} items found</p>
                    </div>
                </div>
            </div>

            {loading ? (
                <div className="flex flex-col items-center justify-center py-20 space-y-4">
                    <Loader2 className="w-8 h-8 text-primary animate-spin" />
                    <p className="text-sm text-muted-foreground italic">Searching for items tagged with #{tag}...</p>
                </div>
            ) : error ? (
                <div className="py-20 text-center space-y-4 text-destructive">
                    <Lock size={32} className="mx-auto" />
                    <h2 className="text-xl font-bold">Search Error</h2>
                    <p>{error}</p>
                </div>
            ) : entities.length > 0 ? (
                <div className="grid gap-3">
                    {entities.map((entity, idx) => (
                        <motion.div
                            key={entity.id}
                            initial={{ opacity: 0, y: 10 }}
                            animate={{ opacity: 1, y: 0 }}
                            transition={{ delay: idx * 0.03 }}
                        >
                            <Link
                                to={`/${entity.type}/${entity.id}`}
                                className="group block bg-card border hover:border-accent hover:shadow-md transition-all rounded-2xl p-4"
                            >
                                <div className="flex items-center justify-between gap-4">
                                    <div className="flex-1 min-w-0 space-y-1.5">
                                        <div className="flex items-center gap-2">
                                            <EntityIcon type={entity.type} size={14} />
                                            <h3 className="font-bold text-lg leading-tight truncate group-hover:text-primary transition-colors">
                                                {entity.title}
                                            </h3>
                                            {entity.starred && <Star size={14} className="text-amber-500 fill-amber-500" />}
                                        </div>

                                        <div className="flex flex-wrap items-center gap-2">
                                            {entity.tags.map(t => (
                                                <TagPill key={t} tag={t} active={t === tag || t === `#${tag}`} />
                                            ))}
                                        </div>
                                    </div>

                                    <div className="flex flex-col items-end gap-2 shrink-0">
                                        <TimeToggle createdAt={entity.created_at} updatedAt={entity.updated_at} minimal />
                                        <ChevronRight size={16} className="text-muted-foreground opacity-0 group-hover:opacity-100 -translate-x-2 group-hover:translate-x-0 transition-all" />
                                    </div>
                                </div>
                            </Link>
                        </motion.div>
                    ))}
                </div>
            ) : (
                <div className="py-20 text-center border-2 border-dashed rounded-3xl space-y-4">
                    <div className="text-muted-foreground italic">No items found with tag #{tag}</div>
                    <Link to="/" className="text-sm font-bold text-primary hover:underline block">
                        Go back home
                    </Link>
                </div>
            )}
        </div>
    )
}

