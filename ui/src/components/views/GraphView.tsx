import React, { useState, useEffect, useRef, useMemo, useCallback } from "react"
import { useNavigate } from "react-router-dom"
import { Loader2, ZoomIn, ZoomOut, MousePointer2, Search, X, Settings2, Activity, Tag } from "lucide-react"
import api from "@/lib/api"
import ForceGraph2D from "react-force-graph-2d"


const COLORS = {
    note: "#60a5fa",     // Blue
    bookmark: "#34d399", // Green
    task: "#f59e0b",     // Amber
    default: "#94a3b8",
    link: "rgba(148, 163, 184, 0.2)"
}

export const GraphView: React.FC = () => {
    const [graphData, setGraphData] = useState<any>(null)
    const [loading, setLoading] = useState(true)
    const [hoverNode, setHoverNode] = useState<any>(null)
    const [searchQuery, setSearchQuery] = useState("")
    const [isSearching, setIsSearching] = useState(false)
    const [isSettingsOpen, setIsSettingsOpen] = useState(false)
    const [showEdgeLabels, setShowEdgeLabels] = useState(true)

    // Physics Configuration
    const [repulsion, setRepulsion] = useState(-2400)
    const [linkDistance, setLinkDistance] = useState(180)

    const fgRef = useRef<any>(null)
    const navigate = useNavigate()

    useEffect(() => {
        const fetchData = () => {
            setLoading(true)
            api.get("/graph")
                .then(res => {
                    setGraphData(res.data)
                    setLoading(false)
                })
                .catch(err => {
                    console.error("Failed to load graph", err)
                    setLoading(false)
                })
        }

        fetchData()
        window.addEventListener("znote-token-changed", fetchData)
        return () => window.removeEventListener("znote-token-changed", fetchData)
    }, [])

    const memoizedData = useMemo(() => {
        if (!graphData) return { nodes: [], links: [] }
        return JSON.parse(JSON.stringify(graphData))
    }, [graphData])

    // Update forces when configuration changes
    useEffect(() => {
        if (fgRef.current && !loading) {
            fgRef.current.d3Force('charge').strength(repulsion);
            fgRef.current.d3Force('link').distance(linkDistance);
            fgRef.current.d3ReheatSimulation();
        }
    }, [repulsion, linkDistance, loading]);

    // Track neighbors for highlighting (similar to minne logic)
    const neighborsData = useMemo(() => {
        if (!memoizedData) return { neighbors: new Map(), nodeLinks: new Map() }
        const neighbors = new Map<string, Set<string>>()
        const nodeLinks = new Map<string, Set<any>>()

        memoizedData.links.forEach((link: any) => {
            const s = typeof link.source === 'object' ? link.source.id : link.source
            const t = typeof link.target === 'object' ? link.target.id : link.target

            if (!neighbors.has(s)) neighbors.set(s, new Set())
            if (!neighbors.has(t)) neighbors.set(t, new Set())
            neighbors.get(s)!.add(t)
            neighbors.get(t)!.add(s)

            if (!nodeLinks.has(s)) nodeLinks.set(s, new Set())
            if (!nodeLinks.has(t)) nodeLinks.set(t, new Set())
            nodeLinks.get(s)!.add(link)
            nodeLinks.get(t)!.add(link)
        })
        return { neighbors, nodeLinks }
    }, [memoizedData])

    const handleNodeClick = useCallback((node: any) => {
        if (!node) return
        // Pin/Unpin on click like minne
        if (node.fx === undefined || node.fx === null) {
            node.fx = node.x
            node.fy = node.y
        } else {
            node.fx = null
            node.fy = null
        }
    }, [])

    const handleNodeRightClick = useCallback((node: any) => {
        navigate(`/${node.type}/${node.id}`)
    }, [navigate])

    const handleSearch = useCallback((e: React.FormEvent) => {
        e.preventDefault()
        if (!searchQuery || !fgRef.current || !memoizedData) return

        const q = searchQuery.toLowerCase()
        const found = memoizedData.nodes.find((n: any) =>
            (n.name || '').toLowerCase().includes(q) ||
            (n.id || '').toLowerCase().includes(q)
        )

        if (found) {
            fgRef.current.centerAt(found.x, found.y, 1000)
            fgRef.current.zoom(2.5, 1000)
            setHoverNode(found)
        }
    }, [searchQuery, memoizedData])

    const getRadius = (node: any) => {
        const degree = (neighborsData.neighbors.get(node.id)?.size || 0)
        return 5 + Math.sqrt(degree) * 2.5 // minne-inspired scaling
    }

    if (loading) {
        return (
            <div className="w-full h-[calc(100vh-12.5rem)] flex flex-col items-center justify-center bg-muted/5 border rounded-xl overflow-hidden shadow-inner">
                <Loader2 className="w-10 h-10 text-primary/40" />
                <p className="text-[10px] text-muted-foreground/60 mt-4 font-bold uppercase tracking-[0.2em]">
                    Relational Physics Syncing...
                </p>
            </div>
        )
    }

    return (
        <div className="w-full h-[calc(100vh-12.5rem)] relative bg-[#09090b] border rounded-xl overflow-hidden shadow-2xl group flex flex-col">
            {/* Header / Search Overlay */}
            <div className="absolute top-0 left-0 right-0 p-4 flex items-center justify-between pointer-events-none z-20">
                <div className="flex flex-col gap-1 pointer-events-auto">
                    <h2 className="text-sm font-bold tracking-tight text-white/90 flex items-center gap-2">
                        <div className="w-2 h-2 rounded-full bg-primary shadow-[0_0_8px_rgba(var(--primary),0.5)]" />
                        Knowledge Map
                    </h2>
                    <p className="text-[10px] text-white/30 uppercase tracking-[0.1em] font-mono">
                        {memoizedData.nodes.length} Nodes • {memoizedData.links.length} Edges
                    </p>
                </div>

                <div className="flex items-center gap-2 pointer-events-auto">
                    {isSearching && (
                        <form
                            onSubmit={handleSearch}
                            className="relative overflow-hidden"
                        >
                            <input
                                type="text"
                                autoFocus
                                placeholder="Find node..."
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                className="w-full h-10 pl-10 pr-4 bg-black/60 backdrop-blur-2xl border border-white/10 rounded-xl text-xs text-white placeholder:text-white/20 focus:outline-none focus:border-primary/50 font-medium shadow-2xl"
                            />
                            <Search size={14} className="absolute left-3.5 top-1/2 -translate-y-1/2 text-white/30" />
                        </form>
                    )}

                    <button
                        onClick={() => setIsSearching(!isSearching)}
                        className={`p-2.5 rounded-xl backdrop-blur-xl border shadow-lg ${isSearching ? 'bg-primary text-primary-foreground border-primary' : 'bg-black/60 text-white/60 border-white/10 hover:text-white'}`}
                    >
                        {isSearching ? <X size={18} /> : <Search size={18} />}
                    </button>

                    <div className="h-5 w-[1px] bg-white/10 mx-1" />

                    <div className="flex bg-black/60 backdrop-blur-xl border border-white/10 rounded-xl p-1 shadow-lg">
                        <button onClick={() => fgRef.current.zoom(fgRef.current.zoom() * 1.5, 400)} className="p-2 rounded-lg text-white/50 hover:text-white hover:bg-white/5"><ZoomIn size={18} /></button>
                        <button onClick={() => fgRef.current.zoom(fgRef.current.zoom() / 1.5, 400)} className="p-2 rounded-lg text-white/50 hover:text-white hover:bg-white/5"><ZoomOut size={18} /></button>
                        <button onClick={() => fgRef.current.centerAt(0, 0, 400)} className="p-2 rounded-lg text-white/50 hover:text-white hover:bg-white/5"><MousePointer2 size={18} /></button>
                    </div>

                    <div className="h-5 w-[1px] bg-white/10 mx-1" />

                    <button
                        onClick={() => setShowEdgeLabels(!showEdgeLabels)}
                        className={`p-2.5 rounded-xl backdrop-blur-xl border shadow-lg ${showEdgeLabels ? 'bg-primary text-primary-foreground border-primary' : 'bg-black/60 text-white/60 border-white/10 hover:text-white'}`}
                        title="Toggle Edge Labels"
                    >
                        <Tag size={18} />
                    </button>

                    <div className="h-5 w-[1px] bg-white/10 mx-1" />

                    <button
                        onClick={() => setIsSettingsOpen(!isSettingsOpen)}
                        className={`p-2.5 rounded-xl backdrop-blur-xl border shadow-lg ${isSettingsOpen ? 'bg-primary text-primary-foreground border-primary' : 'bg-black/60 text-white/60 border-white/10 hover:text-white'}`}
                    >
                        <Settings2 size={18} />
                    </button>

                    {isSettingsOpen && (
                        <div
                            className="absolute top-[120%] right-0 w-64 bg-black/80 backdrop-blur-3xl border border-white/10 rounded-2xl p-5 shadow-[0_20px_50px_rgba(0,0,0,0.5)] z-30 flex flex-col gap-6"
                        >
                            <div className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <h3 className="text-xs font-bold text-white/80 uppercase tracking-widest flex items-center gap-2">
                                        <Activity size={12} className="text-primary" />
                                        Physics Engine
                                    </h3>
                                    <button onClick={() => setIsSettingsOpen(false)} className="text-white/20 hover:text-white">
                                        <X size={14} />
                                    </button>
                                </div>

                                <div className="space-y-5">
                                    <div className="space-y-3">
                                        <div className="flex justify-between items-center text-[10px] font-bold text-white/40 uppercase tracking-tighter">
                                            <span>Node Repulsion</span>
                                            <span className="text-primary font-mono">{Math.abs(repulsion)}</span>
                                        </div>
                                        <input
                                            type="range"
                                            min="-8000"
                                            max="-500"
                                            step="100"
                                            value={repulsion}
                                            onChange={(e) => setRepulsion(Number(e.target.value))}
                                            className="w-full h-1.5 bg-white/5 rounded-full appearance-none cursor-pointer accent-primary"
                                        />
                                    </div>

                                    <div className="space-y-3">
                                        <div className="flex justify-between items-center text-[10px] font-bold text-white/40 uppercase tracking-tighter">
                                            <span>Link Distance</span>
                                            <span className="text-primary font-mono">{linkDistance}px</span>
                                        </div>
                                        <input
                                            type="range"
                                            min="50"
                                            max="600"
                                            step="10"
                                            value={linkDistance}
                                            onChange={(e) => setLinkDistance(Number(e.target.value))}
                                            className="w-full h-1.5 bg-white/5 rounded-full appearance-none cursor-pointer accent-primary"
                                        />
                                    </div>
                                </div>
                            </div>

                            <div className="pt-4 border-t border-white/5">
                                <button
                                    onClick={() => { setRepulsion(-2400); setLinkDistance(180); }}
                                    className="w-full py-2 bg-white/5 hover:bg-white/10 rounded-lg text-[10px] font-bold text-white/40 hover:text-white uppercase tracking-widest"
                                >
                                    Reset to Default
                                </button>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            <ForceGraph2D
                ref={fgRef}
                graphData={memoizedData}
                backgroundColor="#09090b"
                nodeId="id"
                linkSource="source"
                linkTarget="target"

                // Link Style (Minne-style curved edges)
                linkCurvature={0.2}
                linkDirectionalArrowLength={3}
                linkDirectionalArrowRelPos={1}
                linkDirectionalParticles={1}
                linkDirectionalParticleSpeed={0.005}
                linkWidth={link => {
                    if (!hoverNode) return 1
                    const s = typeof link.source === 'object' ? link.source.id : link.source
                    const t = typeof link.target === 'object' ? link.target.id : link.target
                    return (s === hoverNode.id || t === hoverNode.id) ? 2.5 : 0.5
                }}
                linkColor={link => {
                    if (!hoverNode) return COLORS.link
                    const s = typeof link.source === 'object' ? link.source.id : link.source
                    const t = typeof link.target === 'object' ? link.target.id : link.target
                    return (s === hoverNode.id || t === hoverNode.id) ? 'rgba(255, 255, 255, 0.4)' : 'rgba(255, 255, 255, 0.05)'
                }}

                // Edge Labels
                linkCanvasObjectMode={() => 'after'}
                linkCanvasObject={(link: any, ctx, globalScale) => {
                    if (!showEdgeLabels || !link.label) return

                    const start = link.source
                    const end = link.target
                    if (typeof start !== 'object' || typeof end !== 'object') return

                    // Basic label rendering at midpoint
                    const fontSize = 10 / globalScale
                    ctx.font = `${fontSize}px "IBM Plex Mono", monospace`

                    // Highlight based on hover
                    const isHighlighted = !hoverNode || start.id === hoverNode.id || end.id === hoverNode.id
                    if (!isHighlighted) return

                    const text = link.label
                    const textWidth = ctx.measureText(text).width

                    // Multi-link curve offset handling (simplified)
                    const curvature = link.curvature || 0.2
                    const middlePos = {
                        x: start.x + (end.x - start.x) / 2 + (end.y - start.y) * curvature / 4,
                        y: start.y + (end.y - start.y) / 2 - (end.x - start.x) * curvature / 4
                    }

                    let relAngle = Math.atan2(end.y - start.y, end.x - start.x)

                    // Maintain label vertical orientation (don't let it go upside down)
                    if (relAngle > Math.PI / 2) relAngle -= Math.PI
                    if (relAngle < -Math.PI / 2) relAngle += Math.PI

                    ctx.save()
                    ctx.translate(middlePos.x, middlePos.y)
                    ctx.rotate(relAngle)

                    // Background with rounded corners
                    const bgWidth = textWidth + 8 / globalScale
                    const bgHeight = fontSize + 4 / globalScale
                    ctx.fillStyle = 'rgba(9, 9, 11, 0.95)'
                    ctx.beginPath()
                    ctx.roundRect(-bgWidth / 2, -bgHeight / 2, bgWidth, bgHeight, 4 / globalScale)
                    ctx.fill()

                    // Subtle Border
                    ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)'
                    ctx.lineWidth = 0.5 / globalScale
                    ctx.stroke()

                    // Text
                    ctx.textAlign = 'center'
                    ctx.textBaseline = 'middle'
                    ctx.fillStyle = 'rgba(255, 255, 255, 0.9)'
                    ctx.fillText(text, 0, 0)
                    ctx.restore()
                }}

                // Interactive state
                onNodeHover={node => setHoverNode(node)}
                onNodeClick={handleNodeClick}
                onNodeRightClick={handleNodeRightClick}

                // Node Custom Rendering
                nodeCanvasObject={(node: any, ctx, globalScale) => {
                    const r = getRadius(node)
                    const isHighlighted = !hoverNode || node.id === hoverNode.id || neighborsData.neighbors.get(hoverNode.id)?.has(node.id)
                    const opacity = isHighlighted ? 1 : 0.1

                    // Circle shadow / glow
                    if (isHighlighted && node === hoverNode) {
                        ctx.beginPath()
                        ctx.arc(node.x, node.y, r * 1.4, 0, 2 * Math.PI)
                        ctx.fillStyle = `${COLORS[node.type as keyof typeof COLORS] || COLORS.default}22`
                        ctx.fill()
                    }

                    // Main Circle
                    ctx.beginPath()
                    ctx.arc(node.x, node.y, r, 0, 2 * Math.PI)
                    ctx.fillStyle = `${COLORS[node.type as keyof typeof COLORS] || COLORS.default}${Math.floor(opacity * 255).toString(16).padStart(2, '0')}`
                    ctx.fill()

                    // Stroke for pinned nodes
                    if (node.fx !== undefined && node.fx !== null) {
                        ctx.strokeStyle = `rgba(255, 255, 255, ${opacity})`
                        ctx.lineWidth = 2 / globalScale
                        ctx.stroke()
                    }

                    // Label (Visible on hover or if important)
                    const label = node.name || node.id.slice(0, 8)
                    const fontSize = 12 / globalScale
                    if (isHighlighted && (globalScale > 1 || node === hoverNode)) {
                        ctx.font = `${node === hoverNode ? 'bold' : 'normal'} ${fontSize}px "Inter", sans-serif`
                        ctx.textAlign = 'center'
                        ctx.textBaseline = 'middle'
                        ctx.fillStyle = `rgba(255, 255, 255, ${opacity})`
                        ctx.fillText(label, node.x, node.y + r + 10 / globalScale)
                    }
                }}
            />

            {/* Legend / Info Stats */}
            <div className="absolute bottom-6 left-6 flex flex-col gap-3 z-10 pointer-events-none">
                <div className="p-4 bg-black/60 backdrop-blur-xl rounded-xl border border-white/5 space-y-3 shadow-2xl pointer-events-auto">
                    <h3 className="text-[9px] font-bold text-white/30 uppercase tracking-[0.2em]">Graph Directory</h3>
                    <div className="space-y-2">
                        <div className="flex items-center gap-3">
                            <div className="w-2 h-2 rounded-full bg-[#60a5fa] shadow-[0_0_8px_rgba(96,165,250,0.4)]" />
                            <span className="text-[11px] font-medium text-white/70">Knowledge Notes</span>
                        </div>
                        <div className="flex items-center gap-3">
                            <div className="w-2 h-2 rounded-full bg-[#34d399] shadow-[0_0_8px_rgba(52,211,153,0.4)]" />
                            <span className="text-[11px] font-medium text-white/70">Web Bookmarks</span>
                        </div>
                        <div className="flex items-center gap-3">
                            <div className="w-2 h-2 rounded-full bg-[#f59e0b] shadow-[0_0_8px_rgba(245,158,11,0.4)]" />
                            <span className="text-[11px] font-medium text-white/70">Actionable Tasks</span>
                        </div>
                    </div>
                </div>

                <div className="bg-white/5 backdrop-blur-md px-3 py-1.5 rounded-lg border border-white/10">
                    <p className="text-[9px] text-white/40 flex items-center gap-2">
                        <span className="w-1.5 h-1.5 rounded-full bg-white/20" />
                        Right-click to open entity • Drag to pin
                    </p>
                </div>
            </div>
        </div>
    )
}
