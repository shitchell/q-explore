// q-explore history.js - History management

// Update history display in the history tab
function updateHistoryDisplay() {
    const historyList = document.getElementById('history-list');
    if (!historyList) return;

    const history = JSON.parse(localStorage.getItem('q-explore-history') || '[]');

    if (history.length === 0) {
        historyList.innerHTML = '<p class="empty-state">No history yet. Generate some coordinates!</p>';
        return;
    }

    let html = '';
    history.forEach((entry, index) => {
        // Get the first winner for display (attractor by default)
        const type = 'attractor';
        const winner = entry.winners[type] || entry.winners['blind_spot'] || Object.values(entry.winners)[0];

        if (!winner) return;

        const result = winner.result;
        const coords = formatCoords(result.coords.lat, result.coords.lng);
        const date = new Date(entry.timestamp).toLocaleDateString();
        const mode = entry.request.mode === 'flower_power' ? 'Flower' : 'Standard';

        // Convert radius to current display unit
        const radiusDisplay = formatRadius(entry.request.radius);

        html += `
            <div class="history-item" data-index="${index}" data-id="${entry.id}">
                <div class="coords">${coords}</div>
                <div class="meta">
                    ${mode} | ${radiusDisplay} | ${date}
                </div>
            </div>
        `;
    });

    historyList.innerHTML = html;

    // Add click handlers
    historyList.querySelectorAll('.history-item').forEach(item => {
        item.addEventListener('click', () => {
            const index = parseInt(item.dataset.index);
            loadHistoryEntry(index);
        });
    });
}

// Load a history entry and display it on the map
function loadHistoryEntry(index) {
    const history = JSON.parse(localStorage.getItem('q-explore-history') || '[]');
    const entry = history[index];

    if (!entry) return;

    // Restore request parameters
    const req = entry.request;

    // Set location
    setMapCenter(req.lat, req.lng);
    map.setView([req.lat, req.lng], 13);

    // Set radius
    elements.radiusInput.value = req.radius;
    updateRadiusCircle();

    // Set mode
    elements.modeSelect.value = req.mode || 'standard';

    // Build a response object from history
    const response = {
        id: entry.id,
        request: entry.request,
        winners: entry.winners,
        circles: [], // We don't store full circles in history
        metadata: {
            timestamp: entry.timestamp,
        }
    };

    // Display on map
    state.currentResult = response;
    displayResults(response, state.selectedType);
    showResult(response, state.selectedType);

    // Switch to generate tab
    switchTab('generate');
}

// Clear all history
function clearHistory() {
    if (!confirm('Are you sure you want to clear all history?')) {
        return;
    }

    localStorage.removeItem('q-explore-history');
    updateHistoryDisplay();
}

// Delete a specific history entry
function deleteHistoryEntry(id) {
    const history = JSON.parse(localStorage.getItem('q-explore-history') || '[]');
    const filtered = history.filter(entry => entry.id !== id);
    localStorage.setItem('q-explore-history', JSON.stringify(filtered));
    updateHistoryDisplay();
}

// Sync history with server (optional - loads from server API)
async function syncHistoryFromServer() {
    try {
        const response = await fetch('/api/history');
        if (!response.ok) return;

        const data = await response.json();

        // Merge server history with local history
        const localHistory = JSON.parse(localStorage.getItem('q-explore-history') || '[]');
        const localIds = new Set(localHistory.map(e => e.id));

        // Add server entries that aren't in local storage
        data.entries.forEach(entry => {
            if (!localIds.has(entry.response.id)) {
                localHistory.push({
                    id: entry.response.id,
                    timestamp: entry.response.metadata.timestamp,
                    request: entry.response.request,
                    winners: entry.response.winners,
                });
            }
        });

        // Sort by timestamp descending
        localHistory.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));

        // Keep only last 100
        if (localHistory.length > 100) {
            localHistory.length = 100;
        }

        localStorage.setItem('q-explore-history', JSON.stringify(localHistory));
        updateHistoryDisplay();
    } catch (error) {
        console.warn('Failed to sync history from server:', error);
    }
}

// Initialize history tab when document loads
document.addEventListener('DOMContentLoaded', () => {
    // Update history display when switching to history tab
    const historyTab = document.querySelector('[data-tab="history"]');
    if (historyTab) {
        historyTab.addEventListener('click', () => {
            updateHistoryDisplay();
        });
    }

    // Initial load
    updateHistoryDisplay();
});
