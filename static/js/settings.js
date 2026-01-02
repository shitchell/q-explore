// q-explore settings.js - Settings panel management

// Initialize settings functionality
function initSettings() {
    const defaultRadius = document.getElementById('default-radius');
    const defaultBackend = document.getElementById('default-backend');
    const mapProvider = document.getElementById('map-provider');

    // Load saved settings
    loadSettingsToPanel();

    // Add change handlers to auto-save
    if (defaultRadius) {
        defaultRadius.addEventListener('change', saveSettings);
    }
    if (defaultBackend) {
        defaultBackend.addEventListener('change', saveSettings);
    }
    if (mapProvider) {
        mapProvider.addEventListener('change', saveSettings);
    }
}

// Load settings into the settings panel
function loadSettingsToPanel() {
    const settings = JSON.parse(localStorage.getItem('q-explore-settings') || '{}');

    const defaultRadius = document.getElementById('default-radius');
    const defaultBackend = document.getElementById('default-backend');
    const mapProvider = document.getElementById('map-provider');

    if (settings.defaultRadius && defaultRadius) {
        defaultRadius.value = settings.defaultRadius;
    }

    if (settings.defaultBackend && defaultBackend) {
        defaultBackend.value = settings.defaultBackend;
    }

    if (settings.mapProvider && mapProvider) {
        mapProvider.value = settings.mapProvider;
    }
}

// Export history to JSON file
function exportHistory() {
    const history = JSON.parse(localStorage.getItem('q-explore-history') || '[]');

    if (history.length === 0) {
        alert('No history to export');
        return;
    }

    const data = JSON.stringify(history, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    a.download = `q-explore-history-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

// Export history to GPX format
function exportHistoryGpx() {
    const history = JSON.parse(localStorage.getItem('q-explore-history') || '[]');

    if (history.length === 0) {
        alert('No history to export');
        return;
    }

    let gpx = `<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="q-explore" xmlns="http://www.topografix.com/GPX/1/1">
  <metadata>
    <name>q-explore History</name>
    <time>${new Date().toISOString()}</time>
  </metadata>
`;

    history.forEach(entry => {
        const winner = entry.winners['attractor'] || entry.winners['blind_spot'] || Object.values(entry.winners)[0];
        if (!winner) return;

        const result = winner.result;
        const name = `Generation ${entry.id.slice(0, 8)}`;
        const desc = `Mode: ${entry.request.mode || 'standard'}, Radius: ${entry.request.radius}m`;

        gpx += `  <wpt lat="${result.lat}" lon="${result.lng}">
    <name>${name}</name>
    <desc>${desc}</desc>
    <time>${entry.timestamp}</time>
  </wpt>
`;
    });

    gpx += '</gpx>';

    const blob = new Blob([gpx], { type: 'application/gpx+xml' });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    a.download = `q-explore-history-${new Date().toISOString().split('T')[0]}.gpx`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
}

// Import history from JSON file
function importHistory() {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';

    input.onchange = async (e) => {
        const file = e.target.files[0];
        if (!file) return;

        try {
            const text = await file.text();
            const imported = JSON.parse(text);

            if (!Array.isArray(imported)) {
                throw new Error('Invalid format: expected array');
            }

            // Merge with existing history
            const existing = JSON.parse(localStorage.getItem('q-explore-history') || '[]');
            const existingIds = new Set(existing.map(e => e.id));

            imported.forEach(entry => {
                if (entry.id && !existingIds.has(entry.id)) {
                    existing.push(entry);
                }
            });

            // Sort by timestamp descending
            existing.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));

            // Keep only last 100
            if (existing.length > 100) {
                existing.length = 100;
            }

            localStorage.setItem('q-explore-history', JSON.stringify(existing));
            updateHistoryDisplay();

            alert(`Imported ${imported.length} entries`);
        } catch (error) {
            alert(`Import failed: ${error.message}`);
        }
    };

    input.click();
}

// Reset all settings to defaults
function resetSettings() {
    if (!confirm('Are you sure you want to reset all settings to defaults?')) {
        return;
    }

    localStorage.removeItem('q-explore-settings');

    // Reset form elements to defaults
    document.getElementById('default-radius').value = 3000;
    document.getElementById('default-backend').value = 'pseudo';
    document.getElementById('map-provider').value = 'google';

    // Apply to main form as well
    elements.radiusInput.value = 3000;
    elements.backendSelect.value = 'pseudo';
}

// Initialize when document loads
document.addEventListener('DOMContentLoaded', () => {
    initSettings();
});
