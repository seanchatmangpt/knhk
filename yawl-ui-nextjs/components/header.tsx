'use client'

import Link from 'next/link'
import { useState } from 'react'
import { Menu, X, Moon, Sun } from 'lucide-react'
import { useTheme } from 'next-themes'
import { Button } from '@/components/ui/button'

export function Header() {
  const [isOpen, setIsOpen] = useState(false)
  const { setTheme, theme } = useTheme()

  return (
    <header className="border-b bg-background px-6 py-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-8">
          <Link href="/" className="flex items-center gap-2">
            <div className="h-8 w-8 rounded-lg bg-primary text-primary-foreground flex items-center justify-center font-bold">
              Y
            </div>
            <span className="hidden font-bold md:inline">YAWL UI</span>
          </Link>

          <nav className="hidden gap-6 md:flex">
            <Link href="/editor" className="text-sm font-medium hover:text-primary">
              Editor
            </Link>
            <Link href="/workflows" className="text-sm font-medium hover:text-primary">
              Workflows
            </Link>
            <Link href="/monitoring" className="text-sm font-medium hover:text-primary">
              Monitoring
            </Link>
            <Link href="/docs" className="text-sm font-medium hover:text-primary">
              Docs
            </Link>
          </nav>
        </div>

        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
          >
            <Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
            <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
            <span className="sr-only">Toggle theme</span>
          </Button>

          <Button
            variant="ghost"
            size="icon"
            className="md:hidden"
            onClick={() => setIsOpen(!isOpen)}
          >
            {isOpen ? <X className="h-4 w-4" /> : <Menu className="h-4 w-4" />}
          </Button>
        </div>
      </div>

      {isOpen && (
        <nav className="mt-4 flex flex-col gap-2 md:hidden">
          <Link href="/editor" className="px-3 py-2 text-sm font-medium hover:bg-accent rounded">
            Editor
          </Link>
          <Link href="/workflows" className="px-3 py-2 text-sm font-medium hover:bg-accent rounded">
            Workflows
          </Link>
          <Link href="/monitoring" className="px-3 py-2 text-sm font-medium hover:bg-accent rounded">
            Monitoring
          </Link>
          <Link href="/docs" className="px-3 py-2 text-sm font-medium hover:bg-accent rounded">
            Docs
          </Link>
        </nav>
      )}
    </header>
  )
}
