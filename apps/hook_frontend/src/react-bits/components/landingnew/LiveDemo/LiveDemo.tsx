'use client';

import type { CSSProperties } from 'react';

import { FiPlay, FiAlertTriangle } from 'react-icons/fi';
import { useRef, useState, useEffect, useCallback } from 'react';

import { useTranslate } from 'src/locales';
import { primaryColorPresets } from 'src/theme/with-settings';

import { useSettingsContext } from 'src/components/settings';

import {
  type ModelKey,
  getResponseBody,
  getPresetPrompts,
  type PresetPrompt,
  findPresetByPrompt,
  getModelOptionLabel,
} from './live-demo-content';

const DEFAULT_TIME = '11:00:00';
const STREAM_CHAR_DELAY_MS = 15;
const THINK_BLOCK_REGEX = /<think>([\s\S]*?)<\/think>([\s\S]*)/;

type TraceLog = {
  readonly time: string;
  readonly tag: 'client' | 'gateway' | 'upstream' | 'error' | 'success';
  readonly text: string;
};

type SimulationStep = {
  readonly delay: number;
  readonly tag: TraceLog['tag'];
  readonly text: string;
};

type CSSVariableProperties = CSSProperties & {
  readonly [key: `--${string}`]: string;
};

function formatTime(): string {
  return new Date().toTimeString().split(' ')[0] || DEFAULT_TIME;
}

function hexToRgba(hex: string, alpha: number): string {
  const cleanHex = hex.replace('#', '').trim();
  let r = 0;
  let g = 0;
  let b = 0;

  if (cleanHex.length === 3) {
    r = parseInt(cleanHex[0] + cleanHex[0], 16);
    g = parseInt(cleanHex[1] + cleanHex[1], 16);
    b = parseInt(cleanHex[2] + cleanHex[2], 16);
  } else if (cleanHex.length === 6) {
    r = parseInt(cleanHex.slice(0, 2), 16);
    g = parseInt(cleanHex.slice(2, 4), 16);
    b = parseInt(cleanHex.slice(4, 6), 16);
  }

  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function splitResponseContent(responseBody: string) {
  const match = THINK_BLOCK_REGEX.exec(responseBody);

  if (!match) {
    return { thinking: '', content: responseBody };
  }

  return {
    thinking: match[1] || '',
    content: match[2] || '',
  };
}

const LiveDemo = () => {
  const { t } = useTranslate('landing');
  const { state } = useSettingsContext();
  const primaryColor = state.primaryColor;
  const preset = primaryColorPresets[primaryColor] || primaryColorPresets.default;
  const presetPrompts = getPresetPrompts(t);
  const defaultPrompt = presetPrompts[0]?.text ?? '';
  const previousPresetPromptsRef = useRef<readonly PresetPrompt[]>(presetPrompts);
  const timersRef = useRef<ReturnType<typeof setTimeout>[]>([]);

  const [isDark, setIsDark] = useState(false);
  const [model, setModel] = useState<ModelKey>('gpt-4o');
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [simulateFailure, setSimulateFailure] = useState(false);
  const [logs, setLogs] = useState<readonly TraceLog[]>([]);
  const [streamText, setStreamText] = useState('');
  const [thinkingText, setThinkingText] = useState('');
  const [isRunning, setIsRunning] = useState(false);
  const [streamPhase, setStreamPhase] = useState<'idle' | 'thinking' | 'streaming' | 'done'>('idle');

  const clearTimers = useCallback(() => {
    timersRef.current.forEach(clearTimeout);
    timersRef.current = [];
  }, []);

  const scheduleTimer = useCallback((callback: () => void, delay: number) => {
    const timer = setTimeout(callback, delay);
    timersRef.current.push(timer);
    return timer;
  }, []);

  const addLog = useCallback((tag: TraceLog['tag'], text: string) => {
    setLogs((prev) => [...prev, { time: formatTime(), tag, text }]);
  }, []);

  useEffect(() => {
    const checkDark = () => {
      const colorScheme = document.documentElement.getAttribute('data-color-scheme');
      setIsDark(colorScheme === 'dark');
    };

    checkDark();

    const observer = new MutationObserver(checkDark);
    observer.observe(document.documentElement, { attributeFilter: ['data-color-scheme'] });

    return () => observer.disconnect();
  }, []);

  useEffect(() => clearTimers, [clearTimers]);

  useEffect(() => {
    if (isRunning) {
      previousPresetPromptsRef.current = presetPrompts;
      return;
    }

    const previousPreset = findPresetByPrompt(previousPresetPromptsRef.current, prompt);

    if (previousPreset) {
      const nextPreset = presetPrompts.find((item) => item.key === previousPreset.key);

      if (nextPreset && nextPreset.text !== prompt) {
        setPrompt(nextPreset.text);
      }
    } else if (!prompt && defaultPrompt) {
      setPrompt(defaultPrompt);
    }

    previousPresetPromptsRef.current = presetPrompts;
  }, [defaultPrompt, isRunning, presetPrompts, prompt]);

  const styleOverrides: CSSVariableProperties = {
    '--accent-theme': preset.main,
    '--accent-theme-hover': isDark ? (preset.light || preset.main) : (preset.dark || preset.main),
    '--accent-theme-bg': hexToRgba(preset.main, 0.08),
    '--accent-theme-border': hexToRgba(preset.main, 0.25),
    '--accent-theme-contrast': preset.contrastText,
  };

  const handleSend = useCallback(() => {
    if (isRunning) {
      return;
    }

    const promptText = prompt.trim();

    if (!promptText) {
      return;
    }

    setIsRunning(true);
    setLogs([]);
    setStreamText('');
    setThinkingText('');
    setStreamPhase('idle');
    clearTimers();

    const selectedModel = model;
    const presetPrompt = findPresetByPrompt(presetPrompts, promptText);
    const responseBody = getResponseBody(t, selectedModel, promptText, presetPrompt);
    const steps: SimulationStep[] = [
      { delay: 0, tag: 'client', text: t('liveDemo.logs.request', { model: selectedModel }) },
      { delay: 400, tag: 'gateway', text: t('liveDemo.logs.authPassed') },
      { delay: 800, tag: 'gateway', text: t('liveDemo.logs.quotaChecked') },
      { delay: 1200, tag: 'gateway', text: t('liveDemo.logs.routeMatched', { model: selectedModel }) },
    ];

    let currentDelay = 1600;

    if (simulateFailure) {
      steps.push(
        { delay: currentDelay, tag: 'gateway', text: t('liveDemo.logs.primarySelected') },
        { delay: currentDelay + 400, tag: 'upstream', text: t('liveDemo.logs.primaryForwarding') },
        { delay: currentDelay + 1400, tag: 'error', text: t('liveDemo.logs.primaryFailed') },
        { delay: currentDelay + 1800, tag: 'gateway', text: t('liveDemo.logs.failoverTriggered') },
        { delay: currentDelay + 2200, tag: 'gateway', text: t('liveDemo.logs.secondarySelected') },
        { delay: currentDelay + 2600, tag: 'upstream', text: t('liveDemo.logs.secondaryForwarding') },
        { delay: currentDelay + 3200, tag: 'success', text: t('liveDemo.logs.secondarySuccess') }
      );
      currentDelay += 3500;
    } else {
      steps.push(
        { delay: currentDelay, tag: 'gateway', text: t('liveDemo.logs.primarySelected') },
        { delay: currentDelay + 400, tag: 'upstream', text: t('liveDemo.logs.primaryForwarding') },
        { delay: currentDelay + 900, tag: 'success', text: t('liveDemo.logs.primarySuccess') }
      );
      currentDelay += 1200;
    }

    steps.forEach(({ delay, tag, text }) => {
      scheduleTimer(() => addLog(tag, text), delay);
    });

    const { thinking, content } = splitResponseContent(responseBody);
    const totalLength = thinking.length + content.length;
    const streamStartDelay = currentDelay + 200;
    let charIndex = 0;

    const streamNextChar = () => {
      if (thinking && charIndex < thinking.length) {
        setStreamPhase('thinking');
        setThinkingText(thinking.slice(0, charIndex + 1));
        charIndex += 1;
        scheduleTimer(streamNextChar, STREAM_CHAR_DELAY_MS);
        return;
      }

      if (charIndex < totalLength) {
        const contentIndex = charIndex - thinking.length;
        setStreamPhase('streaming');
        setStreamText(content.slice(0, contentIndex + 1));
        charIndex += 1;
        scheduleTimer(streamNextChar, STREAM_CHAR_DELAY_MS);
        return;
      }

      setStreamPhase('done');
      scheduleTimer(() => {
        addLog('success', t('liveDemo.logs.streamComplete'));
        addLog('gateway', t('liveDemo.logs.billing'));
        addLog('gateway', t('liveDemo.logs.walletUpdated'));
        setIsRunning(false);
      }, 200);
    };

    scheduleTimer(streamNextChar, streamStartDelay);
  }, [addLog, clearTimers, isRunning, model, presetPrompts, prompt, scheduleTimer, simulateFailure, t]);

  const handlePromptSelect = useCallback(
    (text: string) => {
      if (isRunning) {
        return;
      }

      setPrompt(text);
    },
    [isRunning]
  );

  return (
    <section id="live-demo" className="ln-demo-section" style={styleOverrides}>
      <div className="ln-demo-inner">
        <h2 className="ln-demo-title">{t('liveDemo.title')}</h2>
        <p className="ln-demo-subtitle">{t('liveDemo.subtitle')}</p>

        <div className="ln-playground">
          <div className="ln-playground-control">
            <div className="ln-control-group">
              <label className="ln-control-label">{t('liveDemo.controls.modelLabel')}</label>
              <select
                className="ln-select"
                value={model}
                onChange={(event) => !isRunning && setModel(event.target.value as ModelKey)}
                disabled={isRunning}
              >
                <option value="gpt-4o">{getModelOptionLabel(t, 'gpt-4o')}</option>
                <option value="claude-3-5-sonnet">{getModelOptionLabel(t, 'claude-3-5-sonnet')}</option>
                <option value="gemini-1-5-pro">{getModelOptionLabel(t, 'gemini-1-5-pro')}</option>
                <option value="deepseek-r1">{getModelOptionLabel(t, 'deepseek-r1')}</option>
              </select>
            </div>

            <div className="ln-control-group">
              <label className="ln-control-label">{t('liveDemo.controls.presetLabel')}</label>
              <div className="ln-prompts-grid">
                {presetPrompts.map((presetPrompt) => (
                  <button
                    key={presetPrompt.key}
                    className={`ln-prompt-btn${prompt === presetPrompt.text ? ' active' : ''}`}
                    onClick={() => handlePromptSelect(presetPrompt.text)}
                    disabled={isRunning}
                  >
                    {presetPrompt.label}
                  </button>
                ))}
              </div>
            </div>

            <div className="ln-control-group">
              <label className="ln-control-label">{t('liveDemo.controls.promptLabel')}</label>
              <textarea
                className="ln-textarea"
                value={prompt}
                onChange={(event) => !isRunning && setPrompt(event.target.value)}
                placeholder={t('liveDemo.controls.promptPlaceholder')}
                disabled={isRunning}
              />
            </div>

            <div className="ln-toggle-wrapper">
              <div className="ln-toggle-info">
                <span className="ln-toggle-title">{t('liveDemo.controls.failoverTitle')}</span>
                <span className="ln-toggle-desc">{t('liveDemo.controls.failoverDescription')}</span>
              </div>
              <label className="ln-switch">
                <input
                  type="checkbox"
                  checked={simulateFailure}
                  onChange={(event) => !isRunning && setSimulateFailure(event.target.checked)}
                  disabled={isRunning}
                />
                <span className="ln-slider" />
              </label>
            </div>

            <button className="ln-send-btn" onClick={handleSend} disabled={isRunning || !prompt.trim()}>
              {isRunning ? (
                <>{t('liveDemo.controls.running')}</>
              ) : (
                <>
                  <FiPlay size={14} /> {t('liveDemo.controls.send')}
                </>
              )}
            </button>
          </div>

          <div className="ln-playground-terminal">
            <div className="ln-terminal-head">
              <div className="ln-terminal-dots">
                <span />
                <span />
                <span />
              </div>
              <span className="ln-terminal-title">{t('liveDemo.terminalTitle')}</span>
            </div>

            <div className="ln-terminal-body">
              {logs.length === 0 && !isRunning && (
                <div style={{ color: 'rgba(255,255,255,0.3)', textAlign: 'center', marginTop: '60px' }}>
                  <FiAlertTriangle size={24} style={{ marginBottom: '8px', opacity: 0.7 }} />
                  <p style={{ margin: 0 }}>{t('liveDemo.emptyState')}</p>
                </div>
              )}

              {logs.map((log, index) => (
                <div key={index} className="ln-trace-log">
                  <span className="ln-trace-time">[{log.time}]</span>
                  <span className="ln-trace-text">
                    <span className={`ln-trace-tag ln-trace-tag--${log.tag}`}>{log.tag.toUpperCase()}</span>
                    {log.text}
                  </span>
                </div>
              ))}

              {(thinkingText || streamText || (isRunning && logs.length >= 5)) && (
                <div className="ln-stream-output">
                  <div className="ln-stream-title">
                    <span>
                      {t('liveDemo.streamTitle')} ({t('liveDemo.streamOutput')})
                    </span>
                    <span style={{ color: 'rgba(56, 242, 178, 0.7)' }}>{model}</span>
                  </div>
                  <div className="ln-stream-text-box">
                    {thinkingText && (
                      <div className="ln-stream-think">
                        &lt;think&gt;
                        {thinkingText}
                        {streamPhase === 'thinking' && <span className="ln-stream-cursor" />}
                        &lt;/think&gt;
                      </div>
                    )}
                    {streamText}
                    {streamPhase === 'streaming' && <span className="ln-stream-cursor" />}
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};

export default LiveDemo;
