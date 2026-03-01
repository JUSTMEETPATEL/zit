# Zit Website

Official website for zit — the AI-powered Git assistant. Built with Next.js 14 and deployed on AWS Amplify.

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Styling**: Tailwind CSS
- **Deployment**: AWS Amplify
- **Font**: Geist (via `next/font`)

## Getting Started

### Prerequisites

- Node.js 18+
- npm, yarn, pnpm, or bun

### Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) to view the site.

### Production Build

```bash
npm run build
npm start
```

## Project Structure

```
app/           # Next.js app router pages
components/    # Reusable React components
lib/           # Utility functions
public/        # Static assets
amplify/       # AWS Amplify configuration
```

## Deployment

The website is automatically deployed via AWS Amplify when changes are pushed to the main branch.

## License

MIT
