# BUSINESS FEASIBILITY (20%)

## Value Proposition (One Sentence)

**"Zit makes Git safe and educational by combining a beautiful terminal UI with AI-powered mentorship — so developers stop Googling errors and start actually understanding Git."**

## Go-to-Market Strategy

### First Customer: Indian Coding Bootcamps & IT Training Institutes

**Why them?**
- They teach thousands of students who struggle with Git
- They need tools that reduce instructor support burden
- They're cost-sensitive — zit is free
- A single instructor adoption means 30-100 student adoptions

**How to reach them?**
1. **Week 1-2**: Publish blog post "How We Built an AI Git Mentor in Rust" on Dev.to — targets developer community
2. **Week 3-4**: Post demo video on Twitter/X and LinkedIn targeting Indian tech communities
3. **Month 2**: Reach out to 10 major bootcamps (Scaler, Masai School, Newton School, Coding Ninjas) offering free workshop integration
4. **Month 3**: Submit talk proposals to PyCon India, RustConf, and local meetup groups
5. **Month 4-6**: GitHub Sponsors campaign + product hunt launch

### Growth Strategy
- **Organic**: Each dev who uses zit recommends it to teammates struggling with Git
- **Content**: YouTube tutorials on "How to resolve merge conflicts with AI" — SEO for common Git problems
- **Open source community**: Accept contributions, build community, create plugins
- **Enterprise pilot**: Offer enterprise features (SSO, audit logs, custom AI models) to 2-3 pilot companies

## Revenue Model Options

### Option 1: Open Core (Recommended)
| Tier | Price | Features |
|------|-------|----------|
| **Free (Open Source)** | $0 | All core features + Ollama (local AI) |
| **Pro** | $8/month | Priority AI (faster models), team analytics, priority support |
| **Enterprise** | $25/user/month | SSO, custom AI models, audit logs, on-prem deployment, SLA |

### Option 2: Managed AI Backend (SaaS)
- Free tier: 100 AI requests/month (enough for solo developers)
- Growth: $5/month for 1,000 requests
- Teams: $15/month for 5,000 requests + team features
- Advantage: No AWS setup required — users get AI instantly

### Option 3: Marketplace Integration
- VS Code extension with zit's AI features
- JetBrains plugin
- Revenue share with IDE marketplaces

## Why Is This Sustainable Beyond the Hackathon?

1. **The problem is permanent**: Git isn't going away. Every new developer will struggle with it. The TAM grows with every CS graduate.
2. **AI costs are dropping**: Claude/GPT API costs have dropped 10x in the last year. Ollama makes local AI free. Our cost structure improves over time.
3. **Open source flywheel**: Contributors add features → more users → more contributors. We've already set up CI/CD, testing infrastructure, and contribution guidelines.
4. **Proven market**: lazygit (56K stars) proves developers want TUI Git tools. GitHub Copilot ($10B ARR) proves developers pay for AI. We're at the intersection.
5. **Low maintenance cost**: Rust binary requires no runtime. Lambda backend is serverless (pay-per-use). Infrastructure is defined as code.

## Likely Judge Question: "How do you make money?"

**Answer**: "We follow the open-core model that's proven by companies like GitLab, Supabase, and PostHog. The core product is free and open source — every developer can use it with local Ollama AI at zero cost. Revenue comes from two streams: First, a managed AI backend service where we host the AI infrastructure so users don't need their own AWS account — $5-15/month based on usage. Second, an enterprise tier with SSO, custom AI models, audit logs, and team analytics at $25/user/month. The free tier is critical because it drives adoption — every developer who loves the free version evangelizes it to their team lead, who then considers the enterprise tier. Our CAC is near-zero because the product is open source and discoverable through GitHub, package managers, and developer communities."
