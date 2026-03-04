import React from "react"
import { CheckCircle2, Circle } from "lucide-react"

interface TaskItemProps {
    text: string
    done?: boolean
}

export const TaskItem: React.FC<TaskItemProps> = ({ text, done }) => (
    <div className="flex items-center gap-3 p-3 rounded-lg border bg-muted/10 group hover:bg-muted/20 cursor-pointer">
        {done ? (
            <CheckCircle2 size={18} className="text-emerald-500 shrink-0" />
        ) : (
            <Circle size={18} className="text-muted-foreground shrink-0 group-hover:text-primary" />
        )}
        <span className={`text-sm ${done ? "text-muted-foreground line-through" : "text-foreground"}`}>
            {text}
        </span>
    </div>
)
