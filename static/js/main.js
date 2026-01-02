// q-explore main.js - Entry point and tab management

// Global state
const state = {
    selectedLat: null,
    selectedLng: null,
    selectedType: 'attractor',
    currentResult: null,
    units: 'metric', // 'metric' or 'imperial'
};

// Conversion constants
const MILES_PER_METER = 0.000621371;
const METERS_PER_MILE = 1609.34;

// DOM elements
const elements = {
    tabs: null,
    tabContents: null,
    locationInput: null,
    latDisplay: null,
    lngDisplay: null,
    radiusInput: null,
    modeSelect: null,
    backendSelect: null,
    typeButtons: null,
    generateBtn: null,
    resultPanel: null,
    resultContent: null,
    hereBtn: null,
    openMapBtn: null,
    copyCoordsBtn: null,
};

// Initialize app
document.addEventListener('DOMContentLoaded', () => {
    initElements();
    initTabs();
    initTypeButtons();
    initLocationInput();
    initMap();
    initGenerate();
    loadSettings();
    checkShareLink();
});

// Handle location input (typing/pasting coordinates)
function initLocationInput() {
    elements.locationInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            parseAndSetLocation(elements.locationInput.value);
        }
    });
}

// Parse coordinates from text input
function parseAndSetLocation(text) {
    if (!text || !text.trim()) return;

    // Try to parse various coordinate formats:
    // "40.7128, -74.0060" or "40.7128,-74.0060" or "40.7128 -74.0060"
    const cleaned = text.trim();

    // Match: number, separator, number (separator can be comma, space, or comma+space)
    const match = cleaned.match(/^(-?\d+\.?\d*)[,\s]+(-?\d+\.?\d*)$/);

    if (match) {
        const lat = parseFloat(match[1]);
        const lng = parseFloat(match[2]);

        if (!isNaN(lat) && !isNaN(lng) && lat >= -90 && lat <= 90 && lng >= -180 && lng <= 180) {
            setMapCenter(lat, lng);
            map.setView([lat, lng], 13);
            return;
        }
    }

    // Could add geocoding here in the future
    alert('Could not parse coordinates. Please use format: lat, lng (e.g., 40.7128, -74.0060)');
}

// Parse URL parameters for share links
function getUrlParams() {
    const params = new URLSearchParams(window.location.search);
    return {
        lat: params.get('lat'),
        lng: params.get('lng'),
        radius: params.get('radius'),
        mode: params.get('mode'),
        backend: params.get('backend'),
        type: params.get('type'),
    };
}

// Check for share link in URL
function checkShareLink() {
    const params = getUrlParams();

    if (params.lat && params.lng) {
        const lat = parseFloat(params.lat);
        const lng = parseFloat(params.lng);

        if (!isNaN(lat) && !isNaN(lng)) {
            // Apply shared parameters
            setMapCenter(lat, lng);
            map.setView([lat, lng], 13);

            if (params.radius) {
                elements.radiusInput.value = params.radius;
                updateRadiusCircle();
            }

            if (params.mode) {
                elements.modeSelect.value = params.mode;
            }

            if (params.backend) {
                elements.backendSelect.value = params.backend;
            }

            if (params.type) {
                // Set active type button
                elements.typeButtons.forEach(btn => {
                    btn.classList.toggle('active', btn.dataset.type === params.type);
                });
                state.selectedType = params.type;
            }

            // Clear the URL params without reloading
            window.history.replaceState({}, document.title, window.location.pathname);
        }
    }
}

// Create a share link for current parameters
async function createShareLink() {
    if (!state.selectedLat || !state.selectedLng) {
        alert('Please select a location first');
        return null;
    }

    const shareData = {
        lat: state.selectedLat,
        lng: state.selectedLng,
        radius: parseFloat(elements.radiusInput.value) || 3000,
        mode: elements.modeSelect.value,
        backend: elements.backendSelect.value,
        type: state.selectedType,
    };

    try {
        const response = await fetch('/api/share', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(shareData),
        });

        if (!response.ok) throw new Error('Failed to create share link');

        const result = await response.json();
        return window.location.origin + window.location.pathname + result.url;
    } catch (error) {
        console.error('Share link error:', error);
        return null;
    }
}

// Copy share link to clipboard
async function handleShareClick() {
    const url = await createShareLink();
    if (url) {
        const success = await copyToClipboard(url);
        if (success) {
            const btn = document.getElementById('share-btn');
            const originalText = btn.textContent;
            btn.textContent = 'Link copied!';
            setTimeout(() => {
                btn.textContent = originalText;
            }, 1500);
        }
    }
}

// Cache DOM elements
function initElements() {
    elements.tabs = document.querySelectorAll('.tab');
    elements.tabContents = document.querySelectorAll('.tab-content');
    elements.locationInput = document.getElementById('location-input');
    elements.latDisplay = document.getElementById('lat-display');
    elements.lngDisplay = document.getElementById('lng-display');
    elements.radiusInput = document.getElementById('radius-input');
    elements.modeSelect = document.getElementById('mode-select');
    elements.backendSelect = document.getElementById('backend-select');
    elements.typeButtons = document.querySelectorAll('.type-btn');
    elements.generateBtn = document.getElementById('generate-btn');
    elements.resultPanel = document.getElementById('result-panel');
    elements.resultContent = document.getElementById('result-content');
    elements.hereBtn = document.getElementById('here-btn');
    elements.openMapBtn = document.getElementById('open-map-btn');
    elements.copyCoordsBtn = document.getElementById('copy-coords-btn');
}

// Tab switching
function initTabs() {
    elements.tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.dataset.tab;
            switchTab(tabName);
        });
    });
}

function switchTab(tabName) {
    // Update tab buttons
    elements.tabs.forEach(tab => {
        tab.classList.toggle('active', tab.dataset.tab === tabName);
    });

    // Update tab contents
    elements.tabContents.forEach(content => {
        const contentId = content.id.replace('-tab', '');
        content.classList.toggle('active', contentId === tabName);
    });
}

// Anomaly type selection
function initTypeButtons() {
    elements.typeButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            elements.typeButtons.forEach(b => b.classList.remove('active'));
            btn.classList.add('active');
            state.selectedType = btn.dataset.type;
        });
    });
}

// Update location display
function updateLocationDisplay(lat, lng) {
    state.selectedLat = lat;
    state.selectedLng = lng;
    elements.latDisplay.textContent = lat.toFixed(6);
    elements.lngDisplay.textContent = lng.toFixed(6);
    elements.locationInput.value = `${lat.toFixed(6)}, ${lng.toFixed(6)}`;
}

// Get current location via IP
async function getCurrentLocation() {
    try {
        const response = await fetch('/api/location');
        if (!response.ok) throw new Error('Failed to get location');
        const data = await response.json();
        return { lat: data.lat, lng: data.lng };
    } catch (error) {
        console.error('Error getting location:', error);
        // Fallback to a default location
        return { lat: 40.7128, lng: -74.0060 }; // NYC
    }
}

// Load settings from localStorage
function loadSettings() {
    const settings = JSON.parse(localStorage.getItem('q-explore-settings') || '{}');

    // Load unit preference
    if (settings.units) {
        state.units = settings.units;
        document.getElementById('unit-select').value = settings.units;
    }

    // Get default radius: localStorage -> data-default-meters fallback
    const defaultRadiusInput = document.getElementById('default-radius');
    const fallbackMeters = parseFloat(defaultRadiusInput.dataset.defaultMeters) || 3000;
    const radiusMeters = settings.defaultRadiusMeters || fallbackMeters;

    // Convert to display unit
    const unitType = state.units === 'imperial' ? 'miles' : 'meters';
    const conversion = UNIT_CONVERSIONS[unitType];
    const displayValue = radiusMeters * conversion.fromMeters;
    const formattedValue = conversion.decimals > 0
        ? displayValue.toFixed(conversion.decimals)
        : Math.round(displayValue);

    // Set Settings default and initialize Generate radius from it
    defaultRadiusInput.value = formattedValue;
    elements.radiusInput.value = formattedValue;

    // Apply labels and steps for current unit system
    applyCurrentUnits();

    if (settings.defaultBackend) {
        elements.backendSelect.value = settings.defaultBackend;
        document.getElementById('default-backend').value = settings.defaultBackend;
    }

    if (settings.mapProvider) {
        document.getElementById('map-provider').value = settings.mapProvider;
    }
}

// Save settings to localStorage
function saveSettings() {
    // Get default radius from settings tab and convert to meters for storage
    const defaultRadiusDisplay = parseFloat(document.getElementById('default-radius').value) || 3000;
    const defaultRadiusMeters = state.units === 'imperial'
        ? defaultRadiusDisplay * METERS_PER_MILE
        : defaultRadiusDisplay;

    const settings = {
        units: state.units,
        defaultRadiusMeters: defaultRadiusMeters,
        defaultBackend: document.getElementById('default-backend').value,
        mapProvider: document.getElementById('map-provider').value,
    };
    localStorage.setItem('q-explore-settings', JSON.stringify(settings));
}

// Format coordinates for display
function formatCoords(lat, lng) {
    return `${lat.toFixed(6)}, ${lng.toFixed(6)}`;
}

// Get map URL based on settings
function getMapUrl(lat, lng) {
    const provider = document.getElementById('map-provider').value;

    switch (provider) {
        case 'google':
            return `https://www.google.com/maps/@${lat},${lng},15z`;
        case 'openstreetmap':
            return `https://www.openstreetmap.org/#map=18/${lat}/${lng}`;
        case 'apple':
            return `https://maps.apple.com/?ll=${lat},${lng}`;
        default:
            return `https://www.google.com/maps/@${lat},${lng},15z`;
    }
}

// Copy text to clipboard
async function copyToClipboard(text) {
    try {
        await navigator.clipboard.writeText(text);
        return true;
    } catch (error) {
        console.error('Failed to copy:', error);
        return false;
    }
}

// Conversion rates: how to convert FROM meters TO the target unit
const UNIT_CONVERSIONS = {
    meters: { fromMeters: 1, toMeters: 1, step: 100, decimals: 0 },
    miles: { fromMeters: MILES_PER_METER, toMeters: METERS_PER_MILE, step: 0.1, decimals: 1 },
    feet: { fromMeters: 3.28084, toMeters: 0.3048, step: 100, decimals: 0 },
    kilometers: { fromMeters: 0.001, toMeters: 1000, step: 0.1, decimals: 1 },
};

// Handle unit change from settings
function handleUnitChange() {
    const unitSelect = document.getElementById('unit-select');
    const newUnit = unitSelect.value;
    const oldUnit = state.units;

    if (newUnit === oldUnit) return;

    state.units = newUnit;
    updateAllUnits(oldUnit, newUnit);
    saveSettings();
    updateRadiusCircle();
}

// Update all unit-labeled elements and unit-value inputs
function updateAllUnits(fromSystem, toSystem) {
    // Update all labels
    document.querySelectorAll('.unit-label').forEach(label => {
        const newUnitType = label.dataset[toSystem];
        if (newUnitType) {
            label.textContent = newUnitType;
        }
    });

    // Update all values
    document.querySelectorAll('.unit-value').forEach(input => {
        const fromUnitType = input.dataset[fromSystem];
        const toUnitType = input.dataset[toSystem];

        if (fromUnitType && toUnitType) {
            const currentValue = parseFloat(input.value) || 0;

            // Convert: current value -> meters -> new unit
            const valueInMeters = currentValue * UNIT_CONVERSIONS[fromUnitType].toMeters;
            const newValue = valueInMeters * UNIT_CONVERSIONS[toUnitType].fromMeters;

            const decimals = UNIT_CONVERSIONS[toUnitType].decimals;
            input.value = decimals > 0 ? newValue.toFixed(decimals) : Math.round(newValue);
            input.step = UNIT_CONVERSIONS[toUnitType].step;
        }
    });
}

// Apply current unit system to all elements (used on page load)
function applyCurrentUnits() {
    const system = state.units;

    // Update all labels
    document.querySelectorAll('.unit-label').forEach(label => {
        const unitType = label.dataset[system];
        if (unitType) {
            label.textContent = unitType;
        }
    });

    // Update input steps (values are already correct from loadSettings)
    document.querySelectorAll('.unit-value').forEach(input => {
        const unitType = input.dataset[system];
        if (unitType && UNIT_CONVERSIONS[unitType]) {
            input.step = UNIT_CONVERSIONS[unitType].step;
        }
    });
}

// Get radius in meters (for API calls)
function getRadiusInMeters() {
    const displayValue = parseFloat(elements.radiusInput.value) || 3000;
    if (state.units === 'imperial') {
        return displayValue * METERS_PER_MILE;
    }
    return displayValue;
}

// Format radius for display (converts from meters to current unit)
function formatRadius(meters) {
    if (state.units === 'imperial') {
        const miles = (meters * MILES_PER_METER).toFixed(1);
        return `${miles} mi`;
    }
    return `${Math.round(meters).toLocaleString()}m`;
}
