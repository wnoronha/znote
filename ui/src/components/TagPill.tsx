import React from "react"
import { Link } from "react-router-dom"
import { cn } from "@/lib/utils"

interface TagPillProps {
    tag: string
    active?: boolean
    className?: string
}

export const TagPill: React.FC<TagPillProps> = ({ tag, active, className }) => {
    const cleanTag = tag.startsWith('#') ? tag.slice(1) : tag
    
    return (
        <Link 
            to={`/tag/${cleanTag}`}
            onClick={(e) => e.stopPropagation()}
            className={cn(
                "text-xs px-3 py-1 rounded-full border transition-all hover:shadow-sm",
                active 
                    ? "bg-primary/10 border-primary/20 text-primary font-bold shadow-sm" 
                    : "bg-muted/30 text-muted-foreground hover:border-accent hover:text-foreground",
                className
            )}
        >
            #{cleanTag}
        </Link>
    )
}
