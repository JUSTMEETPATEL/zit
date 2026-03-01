#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────
#  zit Demo Repo Setup Script
#  Run this ONCE before recording. It creates a git repo with:
#   • A few committed files (so dashboard looks populated)
#   • Uncommitted / unstaged changes (so staging has something to show)
#   • A second branch ready to merge with a conflict
# ─────────────────────────────────────────────────────────────
set -e

DEMO_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DEMO_DIR"

# ── 1. Init repo ────────────────────────────────────────────
git init
git checkout -b main

# ── 2. First commit — project skeleton ──────────────────────
cat > README.md << 'EOF'
# weather-app
A simple weather dashboard that fetches live forecasts.
EOF

cat > app.py << 'EOF'
import requests

API_URL = "https://api.weather.example.com/v1/forecast"

def get_forecast(city: str) -> dict:
    """Fetch a 5-day forecast for the given city."""
    resp = requests.get(API_URL, params={"city": city})
    resp.raise_for_status()
    return resp.json()

def display(forecast: dict) -> None:
    for day in forecast["days"]:
        print(f"{day['date']:>12}  {day['high']}°/{day['low']}°  {day['summary']}")

if __name__ == "__main__":
    data = get_forecast("Mumbai")
    display(data)
EOF

cat > utils.py << 'EOF'
"""Utility helpers for weather-app."""

def celsius_to_fahrenheit(c: float) -> float:
    return c * 9 / 5 + 32

def format_temp(temp: float, unit: str = "C") -> str:
    return f"{temp:.1f}°{unit}"
EOF

git add -A
git commit -m "feat: initial weather-app skeleton"

# ── 3. Second commit — add config ──────────────────────────
cat > config.toml << 'EOF'
[api]
base_url = "https://api.weather.example.com/v1"
timeout = 30

[display]
units = "metric"
color = true
EOF

git add -A
git commit -m "chore: add config file"

# ── 4. Third commit — add tests ────────────────────────────
cat > test_utils.py << 'EOF'
from utils import celsius_to_fahrenheit, format_temp

def test_celsius_to_fahrenheit():
    assert celsius_to_fahrenheit(0) == 32
    assert celsius_to_fahrenheit(100) == 212

def test_format_temp():
    assert format_temp(25.0) == "25.0°C"
    assert format_temp(77.0, "F") == "77.0°F"
EOF

git add -A
git commit -m "test: add unit tests for utils"

# ── 5. Create a feature branch with a CONFLICTING change ───
git checkout -b feature/dark-mode

# Modify app.py on the feature branch (conflict zone)
cat > app.py << 'EOF'
import requests

API_URL = "https://api.weather.example.com/v2/forecast"

THEME = "dark"

def get_forecast(city: str) -> dict:
    """Fetch a 7-day forecast for the given city (dark-mode branch)."""
    resp = requests.get(API_URL, params={"city": city, "days": 7})
    resp.raise_for_status()
    return resp.json()

def display(forecast: dict) -> None:
    header = "🌙 Dark Mode Forecast" if THEME == "dark" else "☀️ Forecast"
    print(header)
    for day in forecast["days"]:
        print(f"  {day['date']:>12}  {day['high']}°/{day['low']}°  {day['summary']}")

if __name__ == "__main__":
    data = get_forecast("Mumbai")
    display(data)
EOF

git add -A
git commit -m "feat: add dark-mode theme to forecast display"

# ── 6. Go back to main and make a DIFFERENT change ─────────
git checkout main

# Modify app.py on main (this will conflict with feature/dark-mode)
cat > app.py << 'EOF'
import requests

API_URL = "https://api.weather.example.com/v2/forecast"

UNITS = "metric"

def get_forecast(city: str) -> dict:
    """Fetch a 7-day forecast with metric units (main branch)."""
    resp = requests.get(API_URL, params={"city": city, "days": 7, "units": UNITS})
    resp.raise_for_status()
    return resp.json()

def display(forecast: dict) -> None:
    print("📊 Weather Forecast")
    for day in forecast["days"]:
        print(f"  {day['date']:>12}  {day['high']}°/{day['low']}°  {day['summary']}")

if __name__ == "__main__":
    data = get_forecast("Mumbai")
    display(data)
EOF

git add -A
git commit -m "feat: add metric units support"

# ── 7. Create UNSTAGED changes (for staging demo) ──────────
cat >> utils.py << 'EOF'

def wind_chill(temp: float, wind_speed: float) -> float:
    """Calculate wind chill factor."""
    return 13.12 + 0.6215 * temp - 11.37 * wind_speed**0.16 + 0.3965 * temp * wind_speed**0.16
EOF

cat >> README.md << 'EOF'

## Features
- 5-day and 7-day forecasts
- Metric and imperial unit support
- Wind chill calculations
EOF

# Create a brand new untracked file
cat > notes.txt << 'EOF'
TODO:
- Add hourly forecast
- Add rain probability chart
- Integrate with notification service
EOF

echo ""
echo "══════════════════════════════════════════════════"
echo "  ✅  Demo repo ready!                          "
echo "══════════════════════════════════════════════════"
echo ""
echo "  Current branch : main"
echo "  Unstaged changes: utils.py, README.md"
echo "  Untracked files : notes.txt"
echo "  Conflict branch : feature/dark-mode"
echo ""
echo "  To start demo:  cd $(pwd) && zit"
echo ""
