import React, { useState, useEffect } from "react"
import ReactMarkdown from "react-markdown"
import remarkGfm from "remark-gfm"
import remarkMath from "remark-math"
import remarkEmoji from "remark-emoji"
import remarkBreaks from "remark-breaks"
import rehypeKatex from "rehype-katex"
import rehypeRaw from "rehype-raw"
import rehypeSlug from "rehype-slug"
import rehypeAutolinkHeadings from "rehype-autolink-headings"
import { Prism as SyntaxHighlighter } from "react-syntax-highlighter"
import { vscDarkPlus } from "react-syntax-highlighter/dist/esm/styles/prism"
import type { ViewMode } from "../../App"
import { Calendar, Share2, CheckCircle2, ExternalLink, ArrowRight, ArrowLeft, Star, IdCard } from "lucide-react"
import { Link } from "react-router-dom"
import api from "@/lib/api"
import { TagPill } from "../TagPill"
import { EntityIcon } from "../EntityIcon"
import { TimeToggle } from "../TimeToggle"
import { EmbeddedEntity } from "../EmbeddedEntity"
import { TaskItem } from "../TaskItem"

interface LinkItem {
    id: string
    title: string
    type: "note" | "bookmark" | "task"
    rel: string
}

interface EntityViewProps {
    id: string
    title: string
    content: string
    tags: string[]
    type: "note" | "bookmark" | "task"
    items?: { text: string; completed: boolean }[]
    url?: string
    createdAt: string
    updatedAt: string
    starred?: boolean
    viewMode: ViewMode
}

export const EntityView: React.FC<EntityViewProps> = ({
    id, title, content, tags, type, items, url, createdAt, updatedAt, starred, viewMode
}) => {
    const [links, setLinks] = useState<{ outgoing: LinkItem[], incoming: LinkItem[] }>({ outgoing: [], incoming: [] })

    const [copied, setCopied] = useState(false) // Fixed useState initialization

    useEffect(() => {
        api.get(`/links/${id}`)
            .then(res => setLinks(res.data))
            .catch(err => console.error("Failed to fetch links", err))
    }, [id])

    const copyToClipboard = () => {
        navigator.clipboard.writeText(window.location.href)
        setCopied(true)
        setTimeout(() => setCopied(false), 2000)
    }

    return (
        <article className="space-y-12 animate-in fade-in slide-in-from-bottom-4 duration-500">
            <header className="space-y-6">
                <div className="flex items-center justify-between">
                    <Link
                        to={`/${type}s`}
                        className="flex items-center gap-2 text-xs font-bold text-muted-foreground tracking-widest uppercase hover:text-primary transition-colors"
                    >
                        {type === 'note' && <Calendar size={14} className="text-secondary-foreground" />}
                        {type === 'bookmark' && <Share2 size={14} className="text-secondary-foreground" />}
                        {type === 'task' && <CheckCircle2 size={14} className="text-secondary-foreground" />}
                        <span>{type}</span>
                    </Link>

                    <div className="flex items-center gap-4">
                        <button
                            onClick={copyToClipboard}
                            title="Copy URL"
                            className="flex items-center gap-1.5 text-[10px] font-mono bg-muted/50 px-2 py-1 rounded border hover:border-primary transition-all group relative"
                        >
                            <IdCard size={12} className="opacity-50 group-hover:opacity-100" />
                            <span className="opacity-50 group-hover:opacity-100">{id}</span>
                            {copied && <span className="absolute -top-8 left-1/2 -translate-x-1/2 bg-primary text-primary-foreground text-[10px] px-2 py-1 rounded shadow-xl animate-in zoom-in-50 fade-in duration-200 whitespace-nowrap">URL Copied!</span>}
                        </button>
                    </div>
                </div>

                <div className="space-y-4">
                    <h1 className="text-4xl md:text-5xl font-bold tracking-tighter text-foreground selection:bg-indigo-500 selection:text-white leading-tight flex items-center gap-4">
                        {title}
                        {starred && <Star size={32} className="text-amber-500 fill-amber-500 shrink-0" />}
                    </h1>
                </div>

                <div className="flex flex-wrap items-center gap-4 text-xs font-medium text-muted-foreground">
                    <TimeToggle createdAt={createdAt} updatedAt={updatedAt} />
                    <div className="flex items-center gap-1.5">
                        {tags.map(tag => (
                            <TagPill key={tag} tag={tag} />
                        ))}
                    </div>
                </div>
            </header>



            {content && (
                <section className={viewMode === 'raw' ? "" : "prose prose-slate dark:prose-invert"}>
                    {viewMode === 'raw' ? (
                        <div className="relative group">
                            <div className="absolute right-4 top-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button
                                    onClick={() => navigator.clipboard.writeText(content)}
                                    className="px-3 py-1.5 bg-muted/80 backdrop-blur-sm border rounded-lg text-[10px] font-bold uppercase tracking-widest hover:bg-primary hover:text-primary-foreground transition-all"
                                >
                                    Copy Raw
                                </button>
                            </div>
                            <pre className="p-6 rounded-2xl bg-muted/20 border font-mono text-sm leading-relaxed overflow-x-auto whitespace-pre-wrap break-all min-h-[200px]">
                                {content}
                            </pre>
                        </div>
                    ) : (
                        <ReactMarkdown
                            remarkPlugins={[remarkGfm, remarkMath, remarkEmoji, remarkBreaks]}
                            rehypePlugins={[rehypeRaw, rehypeKatex, rehypeSlug, rehypeAutolinkHeadings]}
                            components={{
                                p({ children, ...props }: any) {
                                    return (
                                        <p {...props}>
                                            {React.Children.map(children, (child) => {
                                                if (typeof child === 'string') {
                                                    // This handles ![[id]] on its own line OR in the middle of text
                                                    // but to look like Obsidian cards, we mainly focus on it being standalone-ish
                                                    const parts = child.split(/(!\[\[[^\]]+\]\])/g)
                                                    return parts.map((part, i) => {
                                                        const match = /^!\[\[([^\]]+)\]\]$/.exec(part)
                                                        if (match) {
                                                            return <EmbeddedEntity key={i} targetId={match[1]} />
                                                        }
                                                        return part
                                                    })
                                                }
                                                return child
                                            })}
                                        </p>
                                    )
                                },
                                code({ node, inline, className, children, ...props }: any) {
                                    const match = /language-(\w+)/.exec(className || '')
                                    return !inline && match ? (
                                        <div className="my-6 rounded-xl border bg-muted/20 overflow-hidden group relative">
                                            <div className="flex items-center justify-between px-4 py-2 border-b bg-muted/40 text-[10px] font-mono uppercase tracking-widest text-muted-foreground">
                                                <span>{match[1]}</span>
                                                <button
                                                    onClick={() => {
                                                        navigator.clipboard.writeText(String(children));
                                                    }}
                                                    className="hover:text-primary transition-colors"
                                                >
                                                    copy
                                                </button>
                                            </div>
                                            <SyntaxHighlighter
                                                style={vscDarkPlus as any}
                                                language={match[1]}
                                                PreTag="div"
                                                customStyle={{
                                                    margin: 0,
                                                    padding: '1.5rem',
                                                    background: 'transparent',
                                                    fontSize: '0.85rem',
                                                    lineHeight: '1.6'
                                                }}
                                                {...props}
                                            >
                                                {String(children).replace(/\n$/, '')}
                                            </SyntaxHighlighter>
                                        </div>
                                    ) : (
                                        <code className={className} {...props}>
                                            {children}
                                        </code>
                                    )
                                }
                            }}
                        >
                            {content}
                        </ReactMarkdown>
                    )}
                </section>
            )}

            {type === 'task' && items && items.length > 0 && (
                <section className="mt-8 space-y-4">
                    <h3 className="text-sm font-bold uppercase tracking-widest text-muted-foreground px-1">Checklist</h3>
                    <div className="space-y-2">
                        {items.map((item: any, idx: number) => (
                            <TaskItem key={idx} text={item.text} done={item.completed} />
                        ))}
                    </div>
                </section>
            )}

            {type === 'bookmark' && url && (
                <section className="mt-8 space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-700 delay-150">
                    <h3 className="text-sm font-bold uppercase tracking-widest text-muted-foreground px-1">Resource</h3>
                    <a
                        href={url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-4 p-4 rounded-xl border bg-muted/20 group hover:bg-muted/40 hover:border-indigo-500/50 transition-all duration-300"
                    >
                        <div className="w-10 h-10 rounded-lg bg-indigo-500/10 flex items-center justify-center text-indigo-500 group-hover:scale-110 transition-transform">
                            <ExternalLink size={20} />
                        </div>
                        <div className="flex-1 min-w-0">
                            <p className="text-sm font-bold text-foreground truncate">{url}</p>
                            <p className="text-[10px] text-muted-foreground uppercase tracking-wider mt-0.5">Open External Link</p>
                        </div>
                    </a>
                </section>
            )}

            <footer className="border-t pt-12 mt-24 space-y-8">
                <div>
                    <h3 className="text-sm font-bold text-muted-foreground uppercase tracking-widest mb-6 flex items-center gap-2">
                        <ArrowRight size={14} /> Outgoing References
                    </h3>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        {links.outgoing.length > 0 ? (
                            links.outgoing.map(link => (
                                <ReferenceItem key={`${link.id}-${link.rel}`} link={link} />
                            ))
                        ) : (
                            <p className="text-sm text-muted-foreground italic px-1">No outgoing references.</p>
                        )}
                    </div>
                </div>

                <div>
                    <h3 className="text-sm font-bold text-muted-foreground uppercase tracking-widest mb-6 flex items-center gap-2">
                        <ArrowLeft size={14} /> Incoming References
                    </h3>
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        {links.incoming.length > 0 ? (
                            links.incoming.map(link => (
                                <ReferenceItem key={`${link.id}-${link.rel}`} link={link} />
                            ))
                        ) : (
                            <p className="text-sm text-muted-foreground italic px-1">No incoming references.</p>
                        )}
                    </div>
                </div>
            </footer>
        </article>
    )
}

const ReferenceItem: React.FC<{ link: LinkItem }> = ({ link }) => {
    return (
        <Link
            to={`/${link.type}/${link.id}`}
            className="group flex flex-col p-4 rounded-xl border bg-muted/20 hover:bg-muted/50 hover:border-accent transition-all space-y-2"
        >
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <EntityIcon type={link.type} size={14} />
                    <span className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                        {link.rel}
                    </span>
                </div>
            </div>
            <h4 className="font-bold text-sm group-hover:text-primary transition-colors truncate">
                {link.title || link.id.slice(0, 8)}
            </h4>
        </Link>
    )
}

