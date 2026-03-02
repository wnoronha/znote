import React from "react"
import { Sidebar } from "./Sidebar"
import { TopNav } from "./TopNav"
import { CommandPalette } from "./CommandPalette"

import type { ViewMode } from "../../App"

interface LayoutProps {
    children: React.ReactNode
    viewMode: ViewMode
    setViewMode: (mode: ViewMode) => void
}

export const Layout: React.FC<LayoutProps> = ({ children, viewMode, setViewMode }) => {
    return (
        <div className="flex h-screen w-full bg-background overflow-hidden font-sans">
            <Sidebar />
            <div className="flex flex-col flex-1 min-w-0">
                <TopNav viewMode={viewMode} setViewMode={setViewMode} />
                <main className="flex-1 overflow-y-auto p-4 md:p-8">
                    <div className="max-w-4xl mx-auto w-full">
                        {children}
                    </div>
                </main>
            </div>
            <CommandPalette />
        </div>
    )
}
