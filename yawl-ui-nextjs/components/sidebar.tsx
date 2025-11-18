'use client'

import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { cn } from '@/lib/utils'
import { FileText, Network, Zap, BarChart3, BookOpen } from 'lucide-react'

const menuItems = [
  {
    label: 'Workflow Editor',
    href: '/editor',
    icon: FileText,
  },
  {
    label: 'Pattern Library',
    href: '/patterns',
    icon: Network,
  },
  {
    label: 'Workflow Library',
    href: '/workflows',
    icon: Zap,
  },
  {
    label: 'Monitoring',
    href: '/monitoring',
    icon: BarChart3,
  },
  {
    label: 'Documentation',
    href: '/docs',
    icon: BookOpen,
  },
]

export function Sidebar() {
  const pathname = usePathname()

  return (
    <aside className="hidden border-r bg-muted/40 md:block w-56">
      <div className="space-y-2 p-4">
        {menuItems.map((item) => {
          const Icon = item.icon
          const isActive = pathname === item.href || pathname?.startsWith(`${item.href}/`)

          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                'flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
                isActive
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
              )}
            >
              <Icon className="h-4 w-4" />
              {item.label}
            </Link>
          )
        })}
      </div>
    </aside>
  )
}
