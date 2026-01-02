// q-explore generate.js - Generation panel logic

// Initialize generate functionality
function initGenerate() {
    elements.generateBtn.addEventListener('click', handleGenerate);
    elements.openMapBtn.addEventListener('click', handleOpenMap);
    elements.copyCoordsBtn.addEventListener('click', handleCopyCoords);
}

// Handle generate button click
async function handleGenerate() {
    // Validate location
    if (!state.selectedLat || !state.selectedLng) {
        alert('Please select a location on the map first (right-click to select)');
        return;
    }

    // Get parameters (radius is always sent in meters to API)
    const params = {
        lat: state.selectedLat,
        lng: state.selectedLng,
        radius: getRadiusInMeters(),
        points: 10000,
        backend: elements.backendSelect.value,
        mode: elements.modeSelect.value,
        include_points: false,
    };

    // Show loading state
    setLoading(true);
    hideResult();

    try {
        const response = await fetch('/api/generate', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(params),
        });

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || 'Generation failed');
        }

        const result = await response.json();
        state.currentResult = result;

        // Display on map
        displayResults(result, state.selectedType);

        // Show result panel
        showResult(result, state.selectedType);

        // Save to history
        saveToHistory(result);

    } catch (error) {
        console.error('Generation error:', error);
        alert(`Error: ${error.message}`);
    } finally {
        setLoading(false);
    }
}

// Set loading state
function setLoading(loading) {
    elements.generateBtn.disabled = loading;
    elements.generateBtn.classList.toggle('loading', loading);
    elements.generateBtn.textContent = loading ? 'Generating...' : 'Generate';
}

// Show result panel
function showResult(response, displayType) {
    const winner = response.winners[displayType];
    if (!winner) {
        elements.resultContent.innerHTML = '<p>No result found for this type</p>';
        elements.resultPanel.classList.remove('hidden');
        return;
    }

    const result = winner.result;
    const lat = result.coords.lat;
    const lng = result.coords.lng;

    let html = `
        <div class="coords">${formatCoords(lat, lng)}</div>
    `;

    if (result.z_score !== undefined && result.z_score !== null) {
        html += `<div class="z-score">Z-score: ${result.z_score.toFixed(2)}</div>`;
    }

    if (result.is_attractor !== undefined) {
        html += `<div class="z-score">Type: ${result.is_attractor ? 'Attractor' : 'Void'}</div>`;
    }

    elements.resultContent.innerHTML = html;
    elements.resultPanel.classList.remove('hidden');
}

// Hide result panel
function hideResult() {
    elements.resultPanel.classList.add('hidden');
}

// Handle open in maps button
function handleOpenMap() {
    if (!state.currentResult) return;

    const winner = state.currentResult.winners[state.selectedType];
    if (!winner) return;

    const result = winner.result;
    const url = getMapUrl(result.coords.lat, result.coords.lng);
    window.open(url, '_blank');
}

// Handle copy coordinates button
async function handleCopyCoords() {
    if (!state.currentResult) return;

    const winner = state.currentResult.winners[state.selectedType];
    if (!winner) return;

    const result = winner.result;
    const text = formatCoords(result.coords.lat, result.coords.lng);

    const success = await copyToClipboard(text);
    if (success) {
        // Brief visual feedback
        const btn = elements.copyCoordsBtn;
        const originalText = btn.textContent;
        btn.textContent = 'Copied!';
        setTimeout(() => {
            btn.textContent = originalText;
        }, 1500);
    }
}

// Save result to history (localStorage for now, could sync with server)
function saveToHistory(result) {
    const history = JSON.parse(localStorage.getItem('q-explore-history') || '[]');

    // Add to beginning
    history.unshift({
        id: result.id,
        timestamp: result.metadata.timestamp,
        request: result.request,
        winners: result.winners,
    });

    // Keep only last 100 entries
    if (history.length > 100) {
        history.pop();
    }

    localStorage.setItem('q-explore-history', JSON.stringify(history));

    // Update history display if on history tab
    if (typeof updateHistoryDisplay === 'function') {
        updateHistoryDisplay();
    }
}

// Update type button selection and redisplay result
document.addEventListener('DOMContentLoaded', () => {
    const typeButtons = document.querySelectorAll('.type-btn');
    typeButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const type = btn.dataset.type;
            state.selectedType = type;

            // If we have a current result, redisplay with new type
            if (state.currentResult) {
                displayResults(state.currentResult, type);
                showResult(state.currentResult, type);
            }
        });
    });
});
