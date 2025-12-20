//! HTML templates for the web UI.
//!
//! Using inline templates for simplicity. In production, consider askama.

/// Render the base HTML layout.
fn base_layout(title: &str, content: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - Polymarket HFT</title>
    <script src="https://unpkg.com/htmx.org@2.0.4"></script>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        .htmx-indicator {{ display: none; }}
        .htmx-request .htmx-indicator {{ display: inline; }}
    </style>
</head>
<body class="bg-gray-900 text-gray-100 min-h-screen">
    <nav class="bg-gray-800 border-b border-gray-700">
        <div class="max-w-7xl mx-auto px-4">
            <div class="flex items-center justify-between h-16">
                <div class="flex items-center space-x-4">
                    <span class="text-xl font-bold text-emerald-400">📊 Polymarket HFT</span>
                    <a href="/metrics" class="px-3 py-2 rounded-lg hover:bg-gray-700 transition">Metrics</a>
                    <a href="/state" class="px-3 py-2 rounded-lg hover:bg-gray-700 transition">State</a>
                    <a href="/config" class="px-3 py-2 rounded-lg hover:bg-gray-700 transition">Config</a>
                </div>
                <div class="text-sm text-gray-400">
                    <span id="clock"></span>
                </div>
            </div>
        </div>
    </nav>
    <main class="max-w-7xl mx-auto px-4 py-8">
        {content}
    </main>
    <script>
        function updateClock() {{
            document.getElementById('clock').textContent = new Date().toLocaleTimeString();
        }}
        setInterval(updateClock, 1000);
        updateClock();
    </script>
</body>
</html>"##
    )
}

/// Render the metrics dashboard page.
pub fn render_metrics_page() -> String {
    let content = r##"
<div class="space-y-6">
    <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">📈 Metrics Dashboard</h1>
        <div class="flex items-center space-x-4">
            <select id="source-filter" class="bg-gray-800 border border-gray-600 rounded-lg px-3 py-2">
                <option value="">All Sources</option>
                <option value="cmc">CoinMarketCap</option>
                <option value="cg">CoinGecko</option>
                <option value="alt">Alternative.me</option>
            </select>
            <select id="time-range" class="bg-gray-800 border border-gray-600 rounded-lg px-3 py-2">
                <option value="1">Last 1 hour</option>
                <option value="6">Last 6 hours</option>
                <option value="24" selected>Last 24 hours</option>
                <option value="168">Last 7 days</option>
            </select>
            <button class="bg-emerald-600 hover:bg-emerald-500 px-4 py-2 rounded-lg transition">
                🔄 Refresh
            </button>
        </div>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div class="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <div class="text-gray-400 text-sm">Fear &amp; Greed Index</div>
            <div class="text-4xl font-bold text-emerald-400 mt-2" id="fng-value">--</div>
            <div class="text-sm text-gray-500 mt-1" id="fng-label">Loading...</div>
        </div>
        <div class="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <div class="text-gray-400 text-sm">BTC Dominance</div>
            <div class="text-4xl font-bold text-blue-400 mt-2" id="btc-dom">--</div>
            <div class="text-sm text-gray-500 mt-1">Market Share</div>
        </div>
        <div class="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <div class="text-gray-400 text-sm">Total Market Cap</div>
            <div class="text-4xl font-bold text-purple-400 mt-2" id="total-mcap">--</div>
            <div class="text-sm text-gray-500 mt-1">USD</div>
        </div>
    </div>

    <div class="bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
        <div class="px-6 py-4 border-b border-gray-700">
            <h2 class="text-xl font-semibold">Recent Metrics</h2>
        </div>
        <div class="overflow-x-auto">
            <table class="w-full">
                <thead class="bg-gray-750">
                    <tr class="text-left text-gray-400 text-sm">
                        <th class="px-6 py-3">Time</th>
                        <th class="px-6 py-3">Source</th>
                        <th class="px-6 py-3">Name</th>
                        <th class="px-6 py-3">Value</th>
                        <th class="px-6 py-3">Labels</th>
                    </tr>
                </thead>
                <tbody id="metrics-table" class="divide-y divide-gray-700">
                    <tr><td colspan="5" class="px-6 py-4 text-center text-gray-500">No data available. Connect to TimescaleDB to see metrics.</td></tr>
                </tbody>
            </table>
        </div>
    </div>
</div>
"##;
    base_layout("Metrics", content)
}

/// Render the state viewer page.
pub fn render_state_page() -> String {
    let content = r##"
<div class="space-y-6">
    <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">🔮 Current State</h1>
        <button class="bg-emerald-600 hover:bg-emerald-500 px-4 py-2 rounded-lg transition">
            🔄 Refresh
        </button>
    </div>

    <div id="state-container" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <div class="bg-gray-800 rounded-xl p-6 border border-gray-700 col-span-full text-center text-gray-500">
            No state data available. Connect to Redis to see current state.
        </div>
    </div>
</div>
"##;
    base_layout("State", content)
}

/// Render the configuration page.
pub fn render_config_page() -> String {
    let content = r##"
<div class="space-y-6">
    <div class="flex items-center justify-between">
        <h1 class="text-3xl font-bold">⚙️ Scrape Configuration</h1>
        <button 
            hx-get="/api/config" 
            hx-target="#jobs-table-body" 
            hx-swap="innerHTML"
            class="bg-emerald-600 hover:bg-emerald-500 px-4 py-2 rounded-lg transition">
            🔄 Reload
        </button>
    </div>

    <div class="bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
        <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
            <h2 class="text-xl font-semibold">Scrape Jobs</h2>
            <button 
                onclick="showAddModal()"
                class="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded-lg text-sm transition">
                + Add Job
            </button>
        </div>
        <div class="overflow-x-auto">
            <table class="w-full">
                <thead class="bg-gray-750 text-left text-gray-400 text-sm">
                    <tr>
                        <th class="px-6 py-3">ID</th>
                        <th class="px-6 py-3">Source</th>
                        <th class="px-6 py-3">Endpoint</th>
                        <th class="px-6 py-3">Schedule</th>
                        <th class="px-6 py-3">Targets</th>
                        <th class="px-6 py-3">Status</th>
                        <th class="px-6 py-3">Actions</th>
                    </tr>
                </thead>
                <tbody id="jobs-table-body" class="divide-y divide-gray-700" 
                       hx-get="/api/config" 
                       hx-trigger="load"
                       hx-swap="innerHTML">
                    <tr><td colspan="7" class="px-6 py-4 text-center text-gray-500">Loading...</td></tr>
                </tbody>
            </table>
        </div>
    </div>

    <div class="bg-gray-800 rounded-xl border border-gray-700 p-6">
        <h3 class="text-lg font-semibold mb-4">💡 Usage Tips</h3>
        <ul class="text-gray-400 text-sm space-y-2">
            <li>• Jobs can be loaded from config files on startup using <code class="bg-gray-700 px-2 py-1 rounded">--config-file</code> or <code class="bg-gray-700 px-2 py-1 rounded">--config-dir</code></li>
            <li>• Config-loaded jobs are inserted only if they don't already exist</li>
            <li>• Use this UI to modify or delete existing jobs</li>
            <li>• Schedule formats: <code class="bg-gray-700 px-1 rounded">5m</code>, <code class="bg-gray-700 px-1 rounded">1h</code>, <code class="bg-gray-700 px-1 rounded">30s</code> for intervals</li>
        </ul>
    </div>
</div>

<!-- Add/Edit Modal -->
<div id="job-modal" class="fixed inset-0 bg-black bg-opacity-50 hidden items-center justify-center z-50">
    <div class="bg-gray-800 rounded-xl border border-gray-700 p-6 w-full max-w-lg mx-4">
        <h3 id="modal-title" class="text-xl font-semibold mb-4">Add Scrape Job</h3>
        <form id="job-form" class="space-y-4">
            <input type="hidden" id="edit-mode" value="create">
            <div>
                <label class="block text-sm text-gray-400 mb-1">Job ID</label>
                <input type="text" id="job-id" required
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:border-emerald-500">
            </div>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm text-gray-400 mb-1">Source</label>
                    <select id="job-source" required
                            class="w-full bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:border-emerald-500">
                        <option value="alternativeme">Alternative.me</option>
                        <option value="coinmarketcap">CoinMarketCap</option>
                        <option value="coingecko">CoinGecko</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm text-gray-400 mb-1">Endpoint</label>
                    <input type="text" id="job-endpoint" required placeholder="get_fear_and_greed"
                           class="w-full bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:border-emerald-500">
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm text-gray-400 mb-1">Schedule Type</label>
                    <select id="job-schedule-type" required
                            class="w-full bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:border-emerald-500">
                        <option value="interval">Interval</option>
                        <option value="cron">Cron</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm text-gray-400 mb-1">Schedule Value</label>
                    <input type="text" id="job-schedule-value" required placeholder="5m"
                           class="w-full bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:border-emerald-500">
                </div>
            </div>
            <div>
                <label class="block text-sm text-gray-400 mb-1">Targets</label>
                <div class="flex gap-4">
                    <label class="flex items-center gap-2">
                        <input type="checkbox" id="target-metrics" checked class="rounded">
                        <span>Metrics</span>
                    </label>
                    <label class="flex items-center gap-2">
                        <input type="checkbox" id="target-state" checked class="rounded">
                        <span>State</span>
                    </label>
                </div>
            </div>
            <div>
                <label class="block text-sm text-gray-400 mb-1">State TTL (seconds, optional)</label>
                <input type="number" id="job-state-ttl" placeholder="900"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg px-3 py-2 focus:outline-none focus:border-emerald-500">
            </div>
            <div class="flex items-center gap-2">
                <input type="checkbox" id="job-enabled" checked class="rounded">
                <label>Enabled</label>
            </div>
            <div class="flex gap-3 pt-4">
                <button type="submit" class="flex-1 bg-emerald-600 hover:bg-emerald-500 px-4 py-2 rounded-lg transition">
                    Save
                </button>
                <button type="button" onclick="hideModal()" class="flex-1 bg-gray-600 hover:bg-gray-500 px-4 py-2 rounded-lg transition">
                    Cancel
                </button>
            </div>
        </form>
    </div>
</div>

<script>
// Render jobs table from API response
htmx.on('htmx:afterSwap', function(evt) {
    if (evt.detail.target.id === 'jobs-table-body') {
        try {
            const jobs = JSON.parse(evt.detail.xhr.responseText);
            if (Array.isArray(jobs) && jobs.length > 0) {
                evt.detail.target.innerHTML = jobs.map(job => `
                    <tr>
                        <td class="px-6 py-4 font-mono text-sm">${job.id}</td>
                        <td class="px-6 py-4">${job.source}</td>
                        <td class="px-6 py-4">${job.endpoint}</td>
                        <td class="px-6 py-4"><span class="bg-gray-700 px-2 py-1 rounded text-xs">${job.schedule_type}: ${job.schedule_value}</span></td>
                        <td class="px-6 py-4">${job.targets.join(', ')}</td>
                        <td class="px-6 py-4">${job.enabled ? '<span class="text-emerald-400">●</span> Active' : '<span class="text-gray-500">○</span> Disabled'}</td>
                        <td class="px-6 py-4">
                            <button onclick='editJob(${JSON.stringify(job)})' class="text-blue-400 hover:text-blue-300 mr-2">Edit</button>
                            <button onclick="deleteJob('${job.id}')" class="text-red-400 hover:text-red-300">Delete</button>
                        </td>
                    </tr>
                `).join('');
            } else if (Array.isArray(jobs) && jobs.length === 0) {
                evt.detail.target.innerHTML = '<tr><td colspan="7" class="px-6 py-8 text-center text-gray-500">No jobs configured. Add a scrape job to start collecting data.</td></tr>';
            }
        } catch(e) { /* Keep original content if not JSON */ }
    }
});

function showAddModal() {
    document.getElementById('modal-title').textContent = 'Add Scrape Job';
    document.getElementById('edit-mode').value = 'create';
    document.getElementById('job-id').value = '';
    document.getElementById('job-id').readOnly = false;
    document.getElementById('job-source').value = 'alternativeme';
    document.getElementById('job-endpoint').value = '';
    document.getElementById('job-schedule-type').value = 'interval';
    document.getElementById('job-schedule-value').value = '5m';
    document.getElementById('target-metrics').checked = true;
    document.getElementById('target-state').checked = true;
    document.getElementById('job-state-ttl').value = '';
    document.getElementById('job-enabled').checked = true;
    document.getElementById('job-modal').classList.remove('hidden');
    document.getElementById('job-modal').classList.add('flex');
}

function editJob(job) {
    document.getElementById('modal-title').textContent = 'Edit Scrape Job';
    document.getElementById('edit-mode').value = 'update';
    document.getElementById('job-id').value = job.id;
    document.getElementById('job-id').readOnly = true;
    document.getElementById('job-source').value = job.source;
    document.getElementById('job-endpoint').value = job.endpoint;
    document.getElementById('job-schedule-type').value = job.schedule_type;
    document.getElementById('job-schedule-value').value = job.schedule_value;
    document.getElementById('target-metrics').checked = job.targets.includes('metrics');
    document.getElementById('target-state').checked = job.targets.includes('state');
    document.getElementById('job-state-ttl').value = job.state_ttl_secs || '';
    document.getElementById('job-enabled').checked = job.enabled;
    document.getElementById('job-modal').classList.remove('hidden');
    document.getElementById('job-modal').classList.add('flex');
}

function hideModal() {
    document.getElementById('job-modal').classList.add('hidden');
    document.getElementById('job-modal').classList.remove('flex');
}

async function deleteJob(id) {
    if (!confirm('Are you sure you want to delete job: ' + id + '?')) return;
    const res = await fetch('/api/config/' + id, { method: 'DELETE' });
    if (res.ok) {
        htmx.trigger('#jobs-table-body', 'htmx:load');
    } else {
        alert('Failed to delete job');
    }
}

document.getElementById('job-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const mode = document.getElementById('edit-mode').value;
    const id = document.getElementById('job-id').value;
    const targets = [];
    if (document.getElementById('target-metrics').checked) targets.push('metrics');
    if (document.getElementById('target-state').checked) targets.push('state');
    
    const data = {
        id: id,
        source: document.getElementById('job-source').value,
        endpoint: document.getElementById('job-endpoint').value,
        params: {},
        targets: targets,
        schedule_type: document.getElementById('job-schedule-type').value,
        schedule_value: document.getElementById('job-schedule-value').value,
        state_ttl_secs: document.getElementById('job-state-ttl').value ? parseInt(document.getElementById('job-state-ttl').value) : null,
        enabled: document.getElementById('job-enabled').checked
    };
    
    const url = mode === 'create' ? '/api/config' : '/api/config/' + id;
    const method = mode === 'create' ? 'POST' : 'PUT';
    
    const res = await fetch(url, {
        method: method,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data)
    });
    
    if (res.ok) {
        hideModal();
        htmx.trigger('#jobs-table-body', 'htmx:load');
    } else {
        const err = await res.json();
        alert('Error: ' + (err.error || 'Failed to save'));
    }
});
</script>
"##;
    base_layout("Config", content)
}
