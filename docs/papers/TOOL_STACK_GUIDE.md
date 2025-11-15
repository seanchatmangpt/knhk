# Complete Tool Stack Guide

## Essential Tools (Non-negotiable)

### CRM & Sales Pipeline

**Pipedrive** - Primary CRM
```
What: Customer relationship management system
Cost: $12-49/month depending on tier (start with $12/month free tier)
Why: Track prospects, meetings, proposals, deals through pipeline
Setup:
  ├─ Create custom fields:
  │   ├─ Paper Title
  │   ├─ Complexity (Silver/Gold/Platinum)
  │   ├─ Paper URL
  │   └─ Estimated value
  ├─ Create pipeline stages:
  │   ├─ Prospect
  │   ├─ Contacted
  │   ├─ Qualified
  │   ├─ Proposed
  │   ├─ Negotiation
  │   ├─ Committed
  │   └─ Delivered
  ├─ Set up automations (mark contacted when email sent, etc.)
  └─ Weekly review of pipeline status

Alternative: HubSpot Free (very capable free tier, easier for beginners)
```

### Email Outreach

**Lemlist** - Cold email automation
```
What: Automated email outreach with tracking
Cost: $40/month
Why: Scale cold outreach, track opens/clicks, automate follow-ups
Setup:
  ├─ Import email list (from Google Scholar, LinkedIn)
  ├─ Create email sequences:
  │   ├─ Initial email (cold)
  │   ├─ Follow-up 1 (7 days)
  │   ├─ Follow-up 2 (14 days)
  │   └─ Final (21 days)
  ├─ Personalize with variables (PI name, research topic, paper title)
  ├─ Track: Open rate, click rate, reply rate
  └─ Optimize based on performance

Alternative: Free = Gmail + templates (less powerful but free)
```

### Project Management

**Linear** - Issue tracking & project management
```
What: Lightweight project management for individual projects
Cost: $7/month per user (free for solo)
Why: Track tasks within each client project, organize milestones
Setup:
  ├─ Create team workspace
  ├─ For each project:
  │   ├─ Create project/issue for each deliverable
  │   ├─ Break down into tasks
  │   ├─ Set due dates (align with 28-day timeline)
  │   ├─ Add checklists (kickoff, design, implementation, etc.)
  │   └─ Track progress weekly
  ├─ Integrate GitHub (link commits to issues)
  └─ Daily standup: Review what's done/in-progress

Alternative: GitHub Issues (free, simpler, integrated with code)
```

### Code Repository

**GitHub** - Source code hosting
```
What: Git repository hosting, CI/CD, automation
Cost: Free (public repos) or $4/user/month (private)
Why: Host code, run tests, manage deployments, version control
Setup for each project:
  ├─ Create private repo (or public if open source)
  ├─ Set up CI/CD with GitHub Actions:
  │   ├─ Run tests on every push
  │   ├─ Check code coverage
  │   ├─ Build Docker image (if applicable)
  │   ├─ Run linter/formatter
  │   └─ Report status on PR
  ├─ Configure branch protection (main branch)
  ├─ Add CI/CD badge to README
  └─ Document setup in README

Secret management:
  ├─ Use GitHub Secrets for API keys
  ├─ Never commit secrets to repo
  └─ Document how to set up secrets
```

### Development Environment

**VS Code** - Code editor
```
Cost: Free
Why: Best-in-class editor, excellent extensions, lightweight
Setup:
  ├─ Extensions:
  │   ├─ Language support (Rust, Python, JavaScript)
  │   ├─ GitLens (Git history)
  │   ├─ Prettier (code formatting)
  │   ├─ ESLint / Clippy (linting)
  │   ├─ GitHub Copilot (optional, $10/month)
  │   └─ Thunder Client (API testing)
  ├─ Keyboard shortcuts (master them!)
  ├─ Theme (Dracula or One Dark Pro)
  └─ Settings sync (GitHub account)

Settings to configure:
  ├─ Format on save (auto-formatting)
  ├─ Auto-save (save frequently)
  ├─ Tab size (2 or 4 spaces)
  └─ Word wrap (for documentation)
```

### Communication

**Gmail** - Email
```
Cost: Free
Why: Primary communication with clients
Setup:
  ├─ Create professional email address
  ├─ Set up email signature
  ├─ Archive old emails (keep inbox clean)
  ├─ Create labels (clients, projects, follow-ups)
  └─ Use Boomerang or scheduled send for timed emails

Integration:
  ├─ Gmail ↔ Pipedrive sync
  ├─ Gmail ↔ Calendar (schedule meetings)
  └─ Gmail ↔ Lemlist (track email campaigns)
```

**Slack** - Optional, team communication
```
Cost: Free tier is fine for solo
Why: Async communication if/when you hire
Skip for solo phase, add when team grows
```

### Calendar & Scheduling

**Google Calendar** - Calendar management
```
Cost: Free (Google account)
Why: Schedule discovery calls, milestones, training sessions
Setup:
  ├─ Create calendars:
  │   ├─ Main calendar (personal blocks)
  │   ├─ Project calendar (deliverable dates)
  │   ├─ Client meetings (calls with clients)
  │   └─ Off-time (vacation)
  ├─ Block time: Deep work, admin, business dev
  ├─ Share project calendar with clients (optional)
  └─ Use for Zoom/Google Meet links

Integration:
  ├─ Gmail ↔ Calendar (create meetings from emails)
  ├─ Pipedrive ↔ Calendar (sync meetings)
  └─ Lemlist ↔ Calendar (track follow-ups)
```

**Calendly** - Scheduling assistant
```
Cost: $10/month (optional)
Why: Let clients book time without back-and-forth
Setup:
  ├─ Create availability (20-min discovery calls only, 2 slots/week)
  ├─ Include Zoom link
  ├─ Customize confirmation email
  ├─ Set timezone (Pasadena = PT)
  └─ Share link in emails: "Let's chat: [calendly link]"

When to use:
  └─ Send after discovery email with "Let's talk?"
```

---

## Financial & Invoicing

### Invoicing

**Stripe** - Payment processing
```
Cost: 2.9% + $0.30 per transaction
Why: Accept credit card payments (optional, many prefer wire/ACH)
Setup:
  ├─ Create Stripe account
  ├─ Set up products (Silver/Gold/Platinum packages)
  ├─ Send invoice links to clients
  └─ Auto-deposit to bank account

Alternative: Request wire transfer or ACH (lower fees)
```

**Wave** - Free accounting software
```
Cost: Free
Why: Invoicing, expense tracking, financial reports
Setup:
  ├─ Create invoice templates (use Wave's defaults)
  ├─ Send invoices via Wave (track paid status)
  ├─ Track expenses (equipment, tools, etc.)
  ├─ Generate P&L statement monthly
  └─ Export for tax preparation

Workflow:
  ├─ Month 1: Send invoices
  ├─ Month-end: Reconcile bank account
  ├─ Month-end: Generate profit/loss report
  └─ Quarterly: File taxes
```

### Accounting & Tax

**QuickBooks Self-Employed** - Optional
```
Cost: $10-15/month
Why: Simplified accounting, tax prep
When to use: Year 2+ when complexity grows
For now: Wave is sufficient
```

---

## Productivity & Monitoring

### Time Tracking (Optional)

**Toggl** - Time tracking
```
Cost: $9/month (optional)
Why: Understand time allocation, hourly rate calculation
Setup:
  ├─ Track time by project + task
  ├─ Categories:
  │   ├─ Project work (billable)
  │   ├─ Sales/BD (non-billable)
  │   ├─ Admin (non-billable)
  │   └─ Learning (non-billable)
  ├─ Weekly review: Check utilization %
  └─ Calculate effective hourly rate

When to use:
  ├─ Track first 3 months to understand patterns
  ├─ Then track selectively (critical phases only)
  └─ Use for pricing feedback (am I pricing correctly?)
```

### Code Quality

**GitHub Actions** (built-in) - CI/CD
```
Cost: Free (generous limits)
Why: Automated testing on every push
Config (.github/workflows/ci.yml):
  ├─ Trigger: On push and PR
  ├─ Run: cargo test (Rust) or pytest (Python)
  ├─ Run: cargo clippy (linting)
  ├─ Run: Coverage check (>90%)
  ├─ Status badge: Add to README
  └─ Slack/email notification on failure

This proves quality to clients automatically.
```

**Coverage.io** - Code coverage reports
```
Cost: Free (for public repos) or $15/month private
Why: Visualize test coverage, highlight gaps
Setup:
  ├─ Add to CI/CD pipeline
  ├─ Generate coverage report
  ├─ Add badge to README
  └─ Track: Target >90%

Integration: GitHub comments on PR show coverage impact
```

---

## Optional Tools (Year 2+)

### Website & Marketing

**Webflow** - Professional website
```
Cost: $12-35/month
Why: Professional portfolio/case studies
When to build: Year 2 (need case studies first)
Content:
  ├─ About page
  ├─ Services (Silver/Gold/Platinum)
  ├─ Case studies (2-3 success stories)
  ├─ Testimonials
  ├─ Pricing
  └─ Contact form
```

**GitHub Pages** - Free website (alternative)
```
Cost: Free
Why: Host static website, linked to your GitHub repos
When to use: MVP early version (Year 1 if needed)
Setup:
  ├─ Create `username.github.io` repo
  ├─ Use Jekyll (GitHub's static site generator)
  ├─ Commit Markdown content
  └─ Free hosting
```

---

## Security & Data

### Password Management

**1Password** - Password manager
```
Cost: $2.99-4.99/month
Why: Secure password storage, team sharing
Setup:
  ├─ Store all client passwords/tokens
  ├─ Share vault items with team (if hiring)
  ├─ Browser extension for auto-fill
  └─ Emergency access protocol

Essentials:
  └─ Master password (strong, memorized)
```

### Backups

**GitHub** - Code backup
```
Why: All code automatically backed up
Procedure:
  ├─ Push to GitHub regularly
  ├─ Ensure all important code is versioned
  └─ Test cloning from fresh clone yearly
```

**Google Drive** - Document backup
```
Cost: Free (15GB) or $2-10/month (storage)
Why: Backup proposals, contracts, financial documents
Setup:
  ├─ Organize:
  │   ├─ /Clients
  │   ├─ /Proposals
  │   ├─ /Contracts
  │   ├─ /Finance
  │   └─ /Admin
  ├─ Enable version history
  └─ Share with business partner (if applicable)
```

---

## Summary: Year 1 Tool Stack

### Minimum Essential (Month 1)

```
┌─────────────────────────────────────────────────────────────┐
│ START HERE - These 5 tools are non-negotiable              │
├─────────────────────────────────────────────────────────────┤
│ 1. GitHub (Free) - Code hosting + CI/CD                     │
│ 2. VS Code (Free) - Development environment                 │
│ 3. Gmail (Free) - Email communication                       │
│ 4. Google Calendar (Free) - Scheduling                      │
│ 5. Wave (Free) - Invoicing & accounting                     │
│                                                              │
│ Monthly cost: $0                                            │
│ Time to setup: ~4-5 hours                                   │
└─────────────────────────────────────────────────────────────┘
```

### Full Launch Stack (Month 1, before first outreach)

```
┌──────────────────────────────────────────────────────────────┐
│ COMPLETE LAUNCH - All tools needed for professional launch  │
├──────────────────────────────────────────────────────────────┤
│ Sales & CRM:                                                │
│   • Pipedrive ($12/month)                                   │
│   • Lemlist ($40/month)                                     │
│   • Calendly ($10/month, optional)                          │
│                                                              │
│ Development:                                                │
│   • GitHub (Free)                                           │
│   • VS Code (Free)                                          │
│   • GitHub Actions CI/CD (Free)                             │
│                                                              │
│ Communication:                                              │
│   • Gmail (Free)                                            │
│   • Google Calendar (Free)                                  │
│                                                              │
│ Project Management:                                         │
│   • Linear ($7/month, optional - use GitHub Issues)        │
│                                                              │
│ Finance:                                                    │
│   • Wave (Free)                                             │
│   • Stripe (2.9% + $0.30 per transaction)                  │
│                                                              │
│ TOTAL MONTHLY: ~$60-70/month                               │
│ ONE-TIME SETUP: 8-10 hours                                 │
└──────────────────────────────────────────────────────────────┘
```

### Year 2 Additions

```
When revenue supports it:
  ├─ Calendly ($10/month) - If not already
  ├─ Webflow ($20/month) - Professional website
  ├─ 1Password ($5/month) - Team password management
  ├─ Toggl ($9/month) - Time tracking
  ├─ Slack ($6.67/user/month) - If hiring
  └─ Professional email (@yourdomain.com)

TOTAL MONTHLY: ~$150-200/month (but revenue is $30K+)
MARGIN: Still 80%+
```

---

## Implementation Timeline

```
WEEK 1: ESSENTIAL TOOLS
  ├─ Day 1: Set up GitHub account + repos
  ├─ Day 2: Install VS Code + extensions
  ├─ Day 3: Configure GitHub Actions CI/CD
  ├─ Day 4: Set up Gmail, Calendar
  ├─ Day 5: Set up Wave invoicing
  └─ Time: ~8-10 hours

WEEK 2: SALES TOOLS
  ├─ Day 1: Set up Pipedrive CRM
  ├─ Day 2: Configure Pipedrive fields & pipeline
  ├─ Day 3: Set up Lemlist
  ├─ Day 4: Create email sequences
  ├─ Day 5: Set up Calendly (optional)
  └─ Time: ~6-8 hours

TOTAL SETUP: ~15-20 hours (one weekend)
COST: $52-62/month

READY FOR: Cold outreach to start!
```

