import React, { useState } from "react"
import { Clock, Calendar } from "lucide-react"
import { formatDistanceToNow } from "date-fns"

interface TimeToggleProps {
    createdAt: string
    updatedAt: string
    minimal?: boolean
}

export const TimeToggle: React.FC<TimeToggleProps> = ({ createdAt, updatedAt, minimal }) => {
    const [showCreatedAt, setShowCreatedAt] = useState(false)

    if (minimal) {
        return (
            <button
                onClick={(e) => {
                    e.preventDefault()
                    setShowCreatedAt(!showCreatedAt)
                }}
                className="flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground hover:text-primary transition-colors bg-muted/20 px-2 py-1 rounded-md"
            >
                {showCreatedAt ? <Calendar size={10} /> : <Clock size={10} />}
                {showCreatedAt ? 'Created' : 'Updated'} {formatDistanceToNow(new Date(showCreatedAt ? createdAt : updatedAt), { addSuffix: true })}
            </button>
        )
    }

    return (
        <button
            onClick={(e) => {
                e.preventDefault()
                setShowCreatedAt(!showCreatedAt)
            }}
            className="flex items-center gap-1.5 px-3 py-1 bg-muted/50 rounded-full border group hover:border-accent hover:text-foreground transition-all cursor-pointer"
        >
            <Clock size={12} className="opacity-50" />
            <span>{showCreatedAt ? 'Created' : 'Updated'} {formatDistanceToNow(new Date(showCreatedAt ? createdAt : updatedAt), { addSuffix: true })}</span>
        </button>
    )
}
