#!/usr/bin/env python3
"""
Brian2 DOCAS Simulation Script
Generated for Agent Society Simulator
"""

import sys
import json
import numpy as np
from brian2 import *

# Preferred units for DOCAS simulation
ms_unit = ms
mV_unit = mV
Hz_unit = Hz

# Neuromodulator neuron equations
DOPAMINE_EQUATIONS = '''

# Dopaminergic neuron - Leaky Integrate-and-Fire with adaptation
dv/dt = (El - v + g_exc * (Ee - v) + g_inh * (Ei - v)) / tau_m : volt
 dg_exc/dt = -g_exc / tau_e : siemens
 dg_inh/dt = -g_inh / tau_i : siemens

# Parameters (VTA/SNc dopaminergic neuron properties)
tau_m = 20*ms : second
El = -55*mV : volt
Ee = 0*mV : volt
Ei = -80*mV : volt
tau_e = 5*ms : second
tau_i = 10*ms : second

# Spike threshold and reset
v_thresh = -45*mV : volt
v_reset = -55*mV : volt
refractory = 2*ms : second

'''

OXYTOCIN_EQUATIONS = '''

# Oxytocin neuron - Sustained activation for social processing
dv/dt = (El - v + I_social) / tau_m : volt
I_social = g_social * (Es - v) * social_input(t, i) : amp
 dg_social/dt = -g_social / tau_social : siemens

# Parameters (hypothalamic oxytocin neuron properties)
tau_m = 50*ms : second
tau_social = 200*ms : second
El = -60*mV : volt
Es = 0*mV : volt
v_thresh = -50*mV : volt
v_reset = -60*mV : volt
refractory = 5*ms : second

'''

CORTISOL_EQUATIONS = '''

# Cortisol/stress neuron - Rapid threat response
dv/dt = (El - v + g_threat * (Ee - v)) / tau_m : volt
 dg_threat/dt = -g_threat / tau_threat : siemens

# Parameters (amygdala/pituitary stress response)
tau_m = 10*ms : second
tau_threat = 20*ms : second
El = -65*mV : volt
Ee = 0*mV : volt
v_thresh = -40*mV : volt  # Lower threshold for rapid response
v_reset = -65*mV : volt
refractory = 1*ms : second

'''

ADRENALINE_EQUATIONS = '''

# Adrenaline neuron - Action/urgency signaling
dv/dt = (El - v + g_action * (Ee - v)) / tau_m : volt
 dg_action/dt = -g_action / tau_action : siemens

# Parameters (locus coeruleus noradrenergic properties)
tau_m = 15*ms : second
tau_action = 30*ms : second
El = -60*mV : volt
Ee = 0*mV : volt
v_thresh = -45*mV : volt
v_reset = -60*mV : volt
refractory = 1.5*ms : second

'''

SEROTONIN_EQUATIONS = '''

# Serotonin neuron - Stabilization and mood regulation
dv/dt = (El - v + g_stab * (Ee - v) + g_inh * (Ei - v)) / tau_m : volt
 dg_stab/dt = -g_stab / tau_stab : siemens
 dg_inh/dt = -g_inh / tau_i : siemens

# Parameters (raphe nuclei serotonergic properties)
tau_m = 100*ms : second
tau_stab = 500*ms : second
tau_i = 50*ms : second
El = -65*mV : volt
Ee = 0*mV : volt
Ei = -80*mV : volt
v_thresh = -50*mV : volt
v_reset = -65*mV : volt
refractory = 10*ms : second

'''

def create_input_stimuli(params):
    '''Create time-varying input stimuli for DOCAS'''
    t_final = params.get('t_final', 1.0) * second
    dt = params.get('dt', 0.001) * second
    
    # Create time array
    t = np.arange(0, float(t_final), float(dt))
    
    # Extract stimulus parameters
    stimuli = params.get('stimuli', {})
    reward = stimuli.get('reward', 0.0)
    social = stimuli.get('social', 0.0)
    threat = stimuli.get('threat', 0.0)
    action = stimuli.get('action', 0.0)
    stability = stimuli.get('stability', 0.0)
    
    # Create time-varying inputs (can be made more complex)
    reward_input = TimedArray(reward * np.ones(len(t)) * Hz, dt=dt)
    social_input = TimedArray(social * np.ones(len(t)) * Hz, dt=dt)
    threat_input = TimedArray(threat * np.ones(len(t)) * Hz, dt=dt)
    action_input = TimedArray(action * np.ones(len(t)) * Hz, dt=dt)
    stability_input = TimedArray(stability * np.ones(len(t)) * Hz, dt=dt)
    
    return reward_input, social_input, threat_input, action_input, stability_input

def simulate_docas(params):
    '''Simulate DOCAS dynamics in Brian2'''
    
    # Set device for performance
    set_device('runtime', build_on_run=False)
    
    # Parameters
    dt = params.get('dt', 0.001) * second
    t_final = params.get('t_final', 1.0) * second
    n_neurons = params.get('n_neurons', 100)
    
    # Create input stimuli
    reward_input, social_input, threat_input, action_input, stability_input = create_input_stimuli(params)
    
    # Create neuron groups for each neuromodulator
    defaultclock.dt = dt
    
    # Dopamine population (VTA-like)
    da_group = NeuronGroup(
        n_neurons,
        DOPAMINE_EQUATIONS,
        threshold='v > v_thresh',
        reset='v = v_reset',
        refractory='refractory',
        method='euler',
        name='dopamine'
    )
    da_group.v = '-55*mV + rand()*5*mV'  # Random initialization
    
    # Oxytocin population (Hypothalamus-like)
    oxt_group = NeuronGroup(
        n_neurons,
        OXYTOCIN_EQUATIONS,
        threshold='v > v_thresh',
        reset='v = v_reset',
        refractory='refractory',
        method='euler',
        name='oxytocin'
    )
    oxt_group.v = '-60*mV + rand()*5*mV'
    
    # Cortisol population (Amygdala/PVN-like)
    cort_group = NeuronGroup(
        n_neurons,
        CORTISOL_EQUATIONS,
        threshold='v > v_thresh',
        reset='v = v_reset',
        refractory='refractory',
        method='euler',
        name='cortisol'
    )
    cort_group.v = '-65*mV + rand()*5*mV'
    
    # Adrenaline population (Locus Coeruleus-like)
    adr_group = NeuronGroup(
        n_neurons,
        ADRENALINE_EQUATIONS,
        threshold='v > v_thresh',
        reset='v = v_reset',
        refractory='refractory',
        method='euler',
        name='adrenaline'
    )
    adr_group.v = '-60*mV + rand()*5*mV'
    
    # Serotonin population (Raphe-like)
    ser_group = NeuronGroup(
        n_neurons,
        SEROTONIN_EQUATIONS,
        threshold='v > v_thresh',
        reset='v = v_reset',
        refractory='refractory',
        method='euler',
        name='serotonin'
    )
    ser_group.v = '-65*mV + rand()*5*mV'
    
    # Create synaptic connections (cross-modulator coupling)
    # Dopamine -> Oxytocin (reward promotes trust)
    da_oxt = Synapses(da_group, oxt_group, 'w : siemens', on_pre='g_exc += w')
    da_oxt.connect(p=0.1)
    da_oxt.w = '0.05*nS + rand()*0.02*nS'
    
    # Cortisol -> Dopamine (stress reduces reward sensitivity)
    cort_da = Synapses(cort_group, da_group, 'w : siemens', on_pre='g_inh += w')
    cort_da.connect(p=0.1)
    cort_da.w = '0.1*nS + rand()*0.05*nS'
    
    # Cortisol -> Adrenaline (stress triggers action)
    cort_adr = Synapses(cort_group, adr_group, 'w : siemens', on_pre='g_exc += w')
    cort_adr.connect(p=0.15)
    cort_adr.w = '0.2*nS + rand()*0.05*nS'
    
    # Oxytocin -> Serotonin (social bonding promotes stability)
    oxt_ser = Synapses(oxt_group, ser_group, 'w : siemens', on_pre='g_exc += w')
    oxt_ser.connect(p=0.05)
    oxt_ser.w = '0.1*nS + rand()*0.02*nS'
    
    # Serotonin -> Cortisol (serotonin dampens stress)
    ser_cort = Synapses(ser_group, cort_group, 'w : siemens', on_pre='g_inh += w')
    ser_cort.connect(p=0.1)
    ser_cort.w = '0.15*nS + rand()*0.05*nS'
    
    # External inputs (simplified - in real implementation, connect to stimuli)
    # These are Poisson inputs representing external drives
    da_input = PoissonInput(da_group, 'g_exc', 10, params.get('stimuli', {}).get('reward', 0.5)*Hz, weight='0.1*nS')
    oxt_input = PoissonInput(oxt_group, 'g_social', 10, params.get('stimuli', {}).get('social', 0.5)*Hz, weight='0.1*nS')
    cort_input = PoissonInput(cort_group, 'g_threat', 10, params.get('stimuli', {}).get('threat', 0.3)*Hz, weight='0.1*nS')
    adr_input = PoissonInput(adr_group, 'g_action', 10, params.get('stimuli', {}).get('action', 0.4)*Hz, weight='0.1*nS')
    ser_input = PoissonInput(ser_group, 'g_stab', 10, params.get('stimuli', {}).get('stability', 0.6)*Hz, weight='0.1*nS')
    
    # Spike monitors
    da_spikes = SpikeMonitor(da_group)
    oxt_spikes = SpikeMonitor(oxt_group)
    cort_spikes = SpikeMonitor(cort_group)
    adr_spikes = SpikeMonitor(adr_group)
    ser_spikes = SpikeMonitor(ser_group)
    
    # State monitors (record from subset of neurons)
    record_indices = [0, 1, 2, 3, 4]  # Record 5 neurons
    da_state = StateMonitor(da_group, 'v', record=record_indices)
    oxt_state = StateMonitor(oxt_group, 'v', record=record_indices)
    cort_state = StateMonitor(cort_group, 'v', record=record_indices)
    adr_state = StateMonitor(adr_group, 'v', record=record_indices)
    ser_state = StateMonitor(ser_group, 'v', record=record_indices)
    
    # Population rate monitors
    da_rate = PopulationRateMonitor(da_group)
    oxt_rate = PopulationRateMonitor(oxt_group)
    cort_rate = PopulationRateMonitor(cort_group)
    adr_rate = PopulationRateMonitor(adr_group)
    ser_rate = PopulationRateMonitor(ser_group)
    
    # Run simulation
    run(t_final)
    
    # Extract results
    timestamps = da_rate.t / ms  # Convert to ms
    
    # Smooth firing rates for continuous representation
    def smooth_rate(rate_monitor, window=10):
        rate = rate_monitor.rate / Hz
        # Simple moving average
        kernel = np.ones(window) / window
        return np.convolve(rate, kernel, mode='same')
    
    result = {
        'timestamps': (timestamps / 1000.0).tolist(),  # Convert to seconds
        'dopamine_activity': smooth_rate(da_rate).tolist(),
        'oxytocin_activity': smooth_rate(oxt_rate).tolist(),
        'cortisol_activity': smooth_rate(cort_rate).tolist(),
        'adrenaline_activity': smooth_rate(adr_rate).tolist(),
        'serotonin_activity': smooth_rate(ser_rate).tolist(),
        'spike_counts': {
            'dopamine': len(da_spikes.t),
            'oxytocin': len(oxt_spikes.t),
            'cortisol': len(cort_spikes.t),
            'adrenaline': len(adr_spikes.t),
            'serotonin': len(ser_spikes.t),
        },
        'dt': float(dt / ms),
        't_final': float(t_final / ms),
    }
    
    return result

if __name__ == '__main__':
    # Read parameters from stdin
    params = json.load(sys.stdin)
    
    # Run simulation
    result = simulate_docas(params)
    
    # Output results
    print(json.dumps(result))
