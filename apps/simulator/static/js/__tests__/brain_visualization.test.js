import { describe, it, expect } from 'vitest';

globalThis.THREE = globalThis.THREE || {
    Color: class {
        constructor(v) { this.v = v; }
        clone() { return new globalThis.THREE.Color(this.v); }
    }
};

const mod = await import('../brain_visualization.js');
const { BRAIN_REGIONS, BrainVisualization } = mod.default || mod;

describe('Exploding Whole-Body Biomarker Dissection (BRAIN_REGIONS)', () => {
    it('defines all cranial and peripheral biomarker organ nodes with non-zero explosion vectors', () => {
        const expectedNodes = [
            'scn_photoreceptor',
            'cardiac_vagus_node',
            'pulmonary_metabolic_node',
            'adrenal_glands_node',
            'hepatic_glucose_node',
            'enteric_gut_node',
            'vascular_endothelial_node',
            'skeletal_muscle_node',
            'cellular_remodeling_node',
            'genomic_follicular_node'
        ];
        for (const key of expectedNodes) {
            expect(BRAIN_REGIONS[key]).toBeDefined();
            const vec = BRAIN_REGIONS[key].explosionVector;
            expect(Array.isArray(vec)).toBe(true);
            expect(Math.hypot(vec[0], vec[1], vec[2])).toBeGreaterThan(0.5);
        }
    });

    it('every region referenced in BrainVisualization.updateFromDocas exists in BRAIN_REGIONS', () => {
        const src = BrainVisualization.prototype.updateFromDocas.toString();
        const keys = [...src.matchAll(/^\s*([a-z0-9_]+):/gm)].map((m) => m[1]);
        expect(keys.length).toBeGreaterThanOrEqual(20);
        for (const k of keys) {
            expect(BRAIN_REGIONS[k]).toBeDefined();
        }
    });
});
