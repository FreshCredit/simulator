/**
 * DOCAS Real-Time Visualization Client Handler
 */
(function() {
    'use strict';
    // Connect to SSE stream
    const evtSource = new EventSource('/api/docas/stream');
    evtSource.onmessage = function(event) {
        const data = JSON.parse(event.data);
        const d = document.getElementById('dopamine');
        if (d) d.textContent = data.dopamine_mean.toFixed(2);
        const o = document.getElementById('oxytocin');
        if (o) o.textContent = data.oxytocin_mean.toFixed(2);
        const c = document.getElementById('cortisol');
        if (c) c.textContent = data.cortisol_mean.toFixed(2);
        const a = document.getElementById('adrenaline');
        if (a) a.textContent = data.adrenaline_mean.toFixed(2);
        const s = document.getElementById('serotonin');
        if (s) s.textContent = data.serotonin_mean.toFixed(2);
    };
})();
