import React, { useState, useEffect, useRef, useMemo } from "react"
import { useNavigate } from "react-router-dom"
import { Loader2, ZoomIn, ZoomOut, MousePointer2 } from "lucide-react"
import api from "@/lib/api"
import ForceGraph2D from "react-force-graph-2d"
import { forceCollide } from "d3-force"

const COLORS = {
    note: "#6366f1",
    bookmark: "#10b981",
    task: "#f59e0b",
    default: "#94a3b8",
    label: "#e2e8f0"
}

export const GraphView: React.FC = () => {
    const [graphData, setGraphData] = useState<any>(null)
    const [loading, setLoading] = useState(true)
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
        // Deep clone and ensure ID consistency
        return JSON.parse(JSON.stringify(graphData))
    }, [graphData])

    useEffect(() => {
        if (fgRef.current && !loading) {
            // Increase repulsion (default is -30)
            fgRef.current.d3Force('charge').strength(-1200);
            // Increase link distance (default is 30)
            fgRef.current.d3Force('link').distance(400);
            // Add some center force to keep it from flying away too far
            fgRef.current.d3Force('center').strength(0.1);
            // Prevent overlapping
            fgRef.current.d3Force('collision', forceCollide(80));
            // Re-heat simulation to apply changes
            fgRef.current.d3ReheatSimulation();
        }
    }, [loading]);

    if (loading) {
        return (
            <div className="w-full h-[calc(100vh-12rem)] flex flex-col items-center justify-center bg-muted/10 border rounded-2xl">
                <Loader2 className="w-8 h-8 text-primary animate-spin" />
                <p className="text-xs text-muted-foreground mt-3 font-medium uppercase tracking-widest text-center">
                    Simulating Physics...
                </p>
            </div>
        )
    }

    return (
        <div className="w-full h-[calc(100vh-12rem)] relative bg-[#0d0d0f] border border-white/5 rounded-2xl overflow-hidden shadow-2xl group">
            <ForceGraph2D
                ref={fgRef}
                graphData={memoizedData}
                nodeId="id"
                linkSource="source"
                linkTarget="target"
                nodeLabel="name"
                linkLabel="label"
                linkDirectionalArrowLength={6}
                linkDirectionalArrowRelPos={1}
                linkColor={() => "rgba(255, 255, 255, 0.4)"}
                linkWidth={1.5}
                onNodeClick={(node: any) => navigate(`/${node.type}/${node.id}`)}
                nodeCanvasObject={(node: any, ctx, globalScale) => {
                    const label = node.name || node.id.slice(0, 8);
                    const tags = node.tags || [];
                    const tagString = tags.map((t: string) => `#${t}`).join(' ');

                    const fontSize = 14 / globalScale;
                    const tagFontSize = 10 / globalScale;

                    ctx.font = `bold ${fontSize}px "IBM Plex Sans", sans-serif`;
                    const nameWidth = ctx.measureText(label).width;

                    ctx.font = `${tagFontSize}px "IBM Plex Mono", monospace`;
                    const tagsWidth = ctx.measureText(tagString).width;

                    const width = Math.max(nameWidth, tagsWidth) + 20 / globalScale;
                    const height = (tags.length > 0 ? 45 : 30) / globalScale;
                    const radius = 8 / globalScale;

                    // Node Background (Rounded Rect)
                    ctx.beginPath();
                    ctx.roundRect(node.x - width / 2, node.y - height / 2, width, height, radius);

                    // Fill with entity color (semi-transparent)
                    const baseColor = COLORS[node.type as keyof typeof COLORS] || COLORS.default;
                    ctx.fillStyle = `${baseColor}dd`;
                    ctx.fill();

                    // Border
                    ctx.strokeStyle = "rgba(255, 255, 255, 0.5)";
                    ctx.lineWidth = 1 / globalScale;
                    ctx.stroke();

                    // Text
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';

                    // Main Title
                    ctx.fillStyle = "#000000"; // Black text for high contrast on lightish backgrounds
                    ctx.font = `bold ${fontSize}px "IBM Plex Sans", sans-serif`;
                    const titleY = tags.length > 0 ? node.y - 6 / globalScale : node.y;
                    ctx.fillText(label, node.x, titleY);

                    // Tags
                    if (tags.length > 0) {
                        ctx.font = `${tagFontSize}px "IBM Plex Mono", monospace`;
                        ctx.fillStyle = "rgba(0, 0, 0, 0.7)";
                        ctx.fillText(tagString, node.x, node.y + 12 / globalScale);
                    }
                }}
                linkCanvasObjectMode={() => 'after'}
                linkCanvasObject={(link: any, ctx, globalScale) => {
                    if (!link.label) return;
                    const fontSize = 12 / globalScale;
                    const start = link.source;
                    const end = link.target;

                    if (typeof start !== 'object' || typeof end !== 'object') return;

                    const textPos = {
                        x: start.x + (end.x - start.x) / 2,
                        y: start.y + (end.y - start.y) / 2
                    };

                    const relAngle = Math.atan2(end.y - start.y, end.x - start.x);

                    ctx.save();
                    ctx.translate(textPos.x, textPos.y);
                    ctx.rotate(relAngle);

                    ctx.font = `${fontSize}px "IBM Plex Mono"`;
                    const textWidth = ctx.measureText(link.label).width;

                    // Background for labels
                    ctx.fillStyle = 'rgba(13, 13, 15, 0.9)';
                    ctx.fillRect(-textWidth / 2 - 4 / globalScale, -fontSize / 2 - 2 / globalScale, textWidth + 8 / globalScale, fontSize + 4 / globalScale);

                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.fillStyle = '#ffffff';
                    ctx.fillText(link.label, 0, 0);
                    ctx.restore();
                }}
            />

            <div className="absolute top-4 right-4 flex flex-col gap-2 z-10">
                <button onClick={() => fgRef.current.zoom(fgRef.current.zoom() * 1.5, 400)} className="p-2 rounded-lg bg-black/40 backdrop-blur-md border border-white/10 text-white/50 hover:text-white"><ZoomIn size={16} /></button>
                <button onClick={() => fgRef.current.zoom(fgRef.current.zoom() / 1.5, 400)} className="p-2 rounded-lg bg-black/40 backdrop-blur-md border border-white/10 text-white/50 hover:text-white"><ZoomOut size={16} /></button>
                <button onClick={() => fgRef.current.centerAt(0, 0, 400)} className="p-2 rounded-lg bg-black/40 backdrop-blur-md border border-white/10 text-white/50 hover:text-white"><MousePointer2 size={16} /></button>
            </div>

            <div className="absolute bottom-6 left-6 flex gap-6 text-[9px] font-bold text-white/30 uppercase tracking-widest bg-black/60 backdrop-blur-xl px-5 py-2.5 rounded-2xl border border-white/5 shadow-xl items-center pointer-events-none">
                <div className="flex items-center gap-2"><div className="w-2 h-2 rounded-full bg-indigo-500 shadow-[0_0_8px_rgba(99,102,241,0.5)]" /> Note</div>
                <div className="flex items-center gap-2"><div className="w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" /> Bookmark</div>
                <div className="flex items-center gap-2"><div className="w-2 h-2 rounded-full bg-amber-500 shadow-[0_0_8px_rgba(245,158,11,0.5)]" /> Task</div>
            </div>

            <div className="absolute top-6 left-6 pointer-events-none">
                <span className="text-[10px] font-bold text-white/20 uppercase tracking-tighter bg-white/5 px-2 py-1 rounded border border-white/10">Explorer Mode: Force-Graph Engine</span>
            </div>
        </div>
    )
}
