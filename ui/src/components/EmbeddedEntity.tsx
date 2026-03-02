import React, { useState, useEffect } from "react"
import api from "@/lib/api"
import { Loader2, AlertCircle } from "lucide-react"
import ReactMarkdown from "react-markdown"
import remarkGfm from "remark-gfm"
import { EntityIcon } from "./EntityIcon"
import { Link } from "react-router-dom"
import { TaskItem } from "./TaskItem"

interface EmbeddedEntityProps {
    targetId: string
}

export const EmbeddedEntity: React.FC<EmbeddedEntityProps> = ({ targetId }) => {
    const [entity, setEntity] = useState<any>(null)
    const [loading, setLoading] = useState(true)
    const [error, setError] = useState(false)

    const [baseId, header] = targetId.split('#')

    useEffect(() => {
        let mounted = true

        const resolveAndFetch = async () => {
            try {
                const res = await api.get(`/query?expr=id:${baseId}`)
                if (mounted) {
                    if (res.data && res.data.length > 0) {
                        const found = res.data[0]
                        const fullRes = await api.get(`/${found.type}/${found.id}`)
                        setEntity({ ...fullRes.data, type: found.type })
                    } else {
                        setError(true)
                    }
                    setLoading(false)
                }
            } catch (err) {
                console.error("Embed fetch failed", err)
                if (mounted) {
                    setError(true)
                    setLoading(false)
                }
            }
        }

        resolveAndFetch()
        return () => { mounted = false }
    }, [baseId])

    const extractSection = (body: string, targetHeader: string): string => {
        const lines = body.split('\n')
        let matching = false
        const out: string[] = []
        let boundLevel = 0

        const targetLower = targetHeader.trim().toLowerCase()

        for (const line of lines) {
            if (line.startsWith('#')) {
                const level = line.match(/^#+/)?.[0].length || 0
                const text = line.slice(level).trim()

                if (matching) {
                    // Stop if we hit a header of same or higher level
                    if (level <= boundLevel) break
                    out.push(line)
                } else if (text.toLowerCase() === targetLower) {
                    matching = true
                    boundLevel = level
                    out.push(line)
                }
            } else if (matching) {
                out.push(line)
            }
        }

        return matching ? out.join('\n').trim() : `Warning: Section not found: ${targetHeader}`
    }

    if (loading) {
        return (
            <div className="my-4 p-4 rounded-xl border border-dashed flex items-center justify-center gap-3 text-muted-foreground animate-pulse">
                <Loader2 size={16} className="animate-spin" />
                <span className="text-xs font-medium uppercase tracking-widest">Loading Embed...</span>
            </div>
        )
    }

    if (error || !entity) {
        return (
            <div className="my-4 p-4 rounded-xl border border-dashed border-destructive/30 bg-destructive/5 flex items-center gap-3 text-destructive">
                <AlertCircle size={16} />
                <span className="text-xs font-bold uppercase tracking-widest">Embed not found: {baseId}</span>
            </div>
        )
    }

    const rawContent = entity.content || entity.description || ""
    const displayContent = header ? extractSection(rawContent, header) : rawContent

    return (
        <div className="my-6 rounded-2xl border bg-muted/5 hover:bg-muted/10 transition-colors overflow-hidden group">
            <div className="flex items-center justify-between px-4 py-2 bg-muted/20 border-b">
                <Link
                    to={`/${entity.type}/${entity.id}${header ? `#${header}` : ''}`}
                    className="flex items-center gap-2 group/link"
                >
                    <EntityIcon type={entity.type} size={14} />
                    <span className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground group-hover/link:text-primary transition-colors">
                        {entity.type}: {entity.title || entity.id.slice(0, 8)} {header && <span className="text-primary/60"># {header}</span>}
                    </span>
                </Link>
                <div className="flex gap-1">
                    {entity.tags?.slice(0, 2).map((t: string) => (
                        <div key={t} className="text-[8px] px-1.5 py-0.5 rounded-full bg-background border text-muted-foreground uppercase font-bold tracking-tighter">
                            {t}
                        </div>
                    ))}
                </div>
            </div>

            <div className="p-6">
                {entity.type === 'task' && !header ? (
                    <div className="space-y-2">
                        {entity.items?.map((item: any, idx: number) => (
                            <TaskItem key={idx} text={item.text} done={item.completed} />
                        ))}
                        {(!entity.items || entity.items.length === 0) && (
                            <p className="text-xs italic text-muted-foreground px-1">No checklist items.</p>
                        )}
                    </div>
                ) : entity.type === 'bookmark' && !header ? (
                    <div className="flex items-center gap-4">
                        <div className="flex-1 min-w-0">
                            <p className="text-sm font-bold text-foreground truncate">{entity.title || entity.url}</p>
                            <p className="text-[10px] text-muted-foreground uppercase tracking-wider mt-0.5 truncate">{entity.url}</p>
                        </div>
                        <a href={entity.url} target="_blank" rel="noopener noreferrer" className="p-2 rounded-lg bg-primary/10 text-primary hover:bg-primary hover:text-primary-foreground transition-all">
                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" /><polyline points="15 3 21 3 21 9" /><line x1="10" x2="21" y1="14" y2="3" /></svg>
                        </a>
                    </div>
                ) : (
                    <div className="prose prose-sm prose-slate dark:prose-invert max-w-none line-clamp-10">
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>
                            {displayContent}
                        </ReactMarkdown>
                    </div>
                )}
            </div>
        </div>
    )
}
