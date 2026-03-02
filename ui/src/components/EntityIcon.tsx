import React from "react"
import { FileText, Bookmark as BookmarkIcon, CheckSquare, Hash } from "lucide-react"

interface EntityIconProps {
    type: "note" | "bookmark" | "task" | string
    size?: number
    className?: string
}

export const EntityIcon: React.FC<EntityIconProps> = ({ type, size = 14, className }) => {
    switch (type) {
        case "note": return <FileText size={size} className={className || "text-indigo-500"} />
        case "bookmark": return <BookmarkIcon size={size} className={className || "text-emerald-500"} />
        case "task": return <CheckSquare size={size} className={className || "text-amber-500"} />
        default: return <Hash size={size} className={className} />
    }
}
