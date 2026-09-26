import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import {
  LayoutDashboard,
  PlusCircle,
  Search,
  GitBranch,
  FileText,
  Info,
  ChevronLeft,
  ChevronRight,
  Cpu,
} from 'lucide-react';

const navItems = [
  { to: '/', icon: LayoutDashboard, label: 'Dashboard' },
  { to: '/investigations/new', icon: PlusCircle, label: 'New Investigation' },
  { to: '/investigations', icon: Search, label: 'Investigations' },
  { to: '/evidence', icon: GitBranch, label: 'Evidence' },
  { to: '/reports', icon: FileText, label: 'Reports' },
  { to: '/about', icon: Info, label: 'About' },
];

interface Props {
  children: React.ReactNode;
}

export function AppShell({ children }: Props) {
  const [collapsed, setCollapsed] = useState(false);
  const location = useLocation();

  return (
    <div className="flex h-screen bg-gray-50 overflow-hidden">
      {/* Sidebar */}
      <aside
        className={`flex flex-col bg-[#1e3a5f] text-white transition-all duration-200 ${
          collapsed ? 'w-16' : 'w-60'
        } shrink-0`}
      >
        {/* Brand */}
        <div className="flex items-center gap-3 px-4 py-4 border-b border-blue-800">
          <Cpu size={24} className="text-blue-300 shrink-0" />
          {!collapsed && (
            <div>
              <div className="font-bold text-sm leading-tight">TraceForge AI</div>
              <div className="text-blue-300 text-xs">ContractGuard</div>
            </div>
          )}
        </div>

        {/* Nav */}
        <nav className="flex-1 py-4 space-y-1 px-2">
          {navItems.map(({ to, icon: Icon, label }) => {
            const active = to === '/' ? location.pathname === '/' : location.pathname.startsWith(to);
            return (
              <Link
                key={to}
                to={to}
                className={`flex items-center gap-3 px-3 py-2.5 rounded-md text-sm font-medium transition-colors ${
                  active
                    ? 'bg-blue-700 text-white'
                    : 'text-blue-100 hover:bg-blue-800 hover:text-white'
                }`}
              >
                <Icon size={18} className="shrink-0" />
                {!collapsed && <span>{label}</span>}
              </Link>
            );
          })}
        </nav>

        {/* Collapse button */}
        <button
          onClick={() => setCollapsed((c) => !c)}
          className="flex items-center justify-center py-3 border-t border-blue-800 text-blue-300 hover:text-white hover:bg-blue-800 transition-colors"
          aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
        >
          {collapsed ? <ChevronRight size={16} /> : <ChevronLeft size={16} />}
        </button>
      </aside>

      {/* Main */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Top nav */}
        <header className="bg-white border-b border-gray-200 px-6 py-3 flex items-center justify-between">
          <div className="text-sm text-gray-500">
            <span className="font-medium text-gray-900">ContractGuard</span>
            <span className="mx-2 text-gray-300">|</span>
            <span>From Bug Report to Verified Fix</span>
          </div>
          <Link
            to="/investigations/new"
            className="inline-flex items-center gap-2 bg-blue-600 text-white px-4 py-1.5 rounded-md text-sm font-medium hover:bg-blue-700 transition-colors"
          >
            <PlusCircle size={16} />
            New Investigation
          </Link>
        </header>

        {/* Content */}
        <main className="flex-1 overflow-y-auto p-6">
          {children}
        </main>
      </div>
    </div>
  );
}
