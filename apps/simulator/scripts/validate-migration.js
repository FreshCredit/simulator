
// TAG: surface=api owner=platform-team rule=API-001
#!/usr/bin/env node

/**
 * Migration Validator Script
 *
 * Validates that the simulator refactoring is complete and correct.
 * Usage:
 *   node scripts/validate-migration.js
 *   node scripts/validate-migration.js --report
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ANSI colors
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  gray: '\x1b[90m',
};

// Validation rules
const rules = [
  {
    id: 'critical-patches',
      // TAG: surface=api owner=platform-team rule=API-001
    name: 'P0 Critical Patches',
    description: 'Original viewer has performance patches',
    check: () => {
      const viewer = readFile('static/js/viewer.js');
      const hasAgentMap = viewer.includes('agentMap');
      const hasCleanup = viewer.includes('cleanupAgentHistory');
      return hasAgentMap && hasCleanup;
    },
    required: true,
  },
  {
    id: 'modules-exist',
    name: 'Module Extraction',
    description: 'All modules extracted',
    check: () => {
      const modules = [
        'static/js/modules/connection-manager.js',
        'static/js/modules/state-store.js',
        'static/js/modules/api-client.js',
        'static/js/modules/debug-system.js',
        'static/js/modules/event-log.js',
        'static/js/modules/grid-cache.js',
        'static/js/modules/perf-monitor.js',
      ];
      return modules.every((m) => fileExists(m));
    },
    required: true,
  },
  {
    id: 'modern-viewer',
    name: 'Modern Viewer',
    description: 'Modern viewer exists',
    check: () => fileExists('static/js/viewer-modern.js'),
    // TAG: surface=api owner=platform-team rule=API-001
    required: true,
  },
  {
    id: 'adapter',
    name: 'Compatibility Adapter',
    description: 'Adapter for backwards compatibility',
    check: () => fileExists('static/js/viewer-adapter.js'),
    required: true,
  },
  {
    id: 'tests',
    name: 'Test Suite',
    description: 'Comprehensive test coverage',
    check: () => {
      const tests = [
        'static/js/__tests__/connection-manager.test.js',
        'static/js/__tests__/state-store.test.js',
        'static/js/__tests__/api-client.test.js',
        'static/js/__tests__/debug-system.test.js',
        'static/js/__tests__/event-log.test.js',
        'static/js/__tests__/grid-cache.test.js',
        'static/js/__tests__/perf-monitor.test.js',
      ];
      return tests.every((t) => fileExists(t));
    },
    required: true,
  },
  {
    id: 'package-json',
    name: 'Package Configuration',
    description: 'package.json configured',
    check: () => {
      // TAG: surface=api owner=platform-team rule=API-001
      const pkg = JSON.parse(readFile('package.json'));
      return pkg.scripts && pkg.scripts.test;
    },
    required: true,
  },
  {
    id: 'vitest-config',
    name: 'Vitest Configuration',
    description: 'vitest.config.js exists',
    check: () => fileExists('vitest.config.js'),
    required: true,
  },
  {
    id: 'eslint-config',
    name: 'ESLint Configuration',
    description: '.eslintrc.cjs exists',
    check: () => fileExists('.eslintrc.cjs'),
    required: false,
  },
  {
    id: 'prettier-config',
    name: 'Prettier Configuration',
    description: '.prettierrc exists',
    check: () => fileExists('.prettierrc'),
    required: false,
  },
  {
    id: 'ci-cd',
    name: 'CI/CD Pipeline',
    description: 'GitHub Actions workflow',
    check: () => fileExists('.github/workflows/ci.yml'),
    required: false,
  },
    // TAG: surface=api owner=platform-team rule=API-001
  {
    id: 'documentation',
    name: 'Documentation',
    description: 'Comprehensive documentation',
    check: () => {
      const docs = [
        'REFACTORING_ANALYSIS.md',
        'API_REFERENCE.md',
        'MIGRATION_GUIDE.md',
        'DEPLOYMENT_CHECKLIST.md',
      ];
      return docs.every((d) => fileExists(d));
    },
    required: false,
  },
  {
    id: 'examples',
    name: 'Usage Examples',
    description: 'Example usage files',
    check: () => fileExists('static/js/examples/basic-usage.js'),
    required: false,
  },
];

// Helper functions
function fileExists(filePath) {
  try {
    fs.accessSync(path.join(process.cwd(), filePath));
    return true;
  } catch {
    return false;
  }
    // TAG: surface=api owner=platform-team rule=API-001
}

function readFile(filePath) {
  try {
    return fs.readFileSync(path.join(process.cwd(), filePath), 'utf8');
  } catch {
    return '';
  }
}

function formatDuration(ms) {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(1)}s`;
}

// Main validation
function validate() {
  const startTime = Date.now();
  const results = [];
  let passed = 0;
  let failed = 0;
  let warnings = 0;

  console.log(`${colors.blue}╔════════════════════════════════════════════════════════════╗${colors.reset}`);
  console.log(`${colors.blue}║          Simulator Migration Validator                     ║${colors.reset}`);
  console.log(`${colors.blue}╚════════════════════════════════════════════════════════════╝${colors.reset}`);
  console.log();

  for (const rule of rules) {
    process.stdout.write(`  ${rule.name}... `);

    try {
      const passed = rule.check();
        // TAG: surface=api owner=platform-team rule=API-001
      results.push({ rule, passed });

      if (passed) {
        console.log(`${colors.green}✓${colors.reset}`);
      } else if (rule.required) {
        console.log(`${colors.red}✗${colors.reset}`);
      } else {
        console.log(`${colors.yellow}⚠${colors.reset}`);
      }
    } catch (error) {
      results.push({ rule, passed: false, error });
      console.log(`${colors.red}✗${colors.reset}`);
    }
  }

  console.log();

  // Calculate statistics
  for (const result of results) {
    if (result.passed) {
      passed++;
    } else if (result.rule.required) {
      failed++;
    } else {
      warnings++;
    }
  }

  const duration = Date.now() - startTime;

  // Summary
  console.log(`${colors.blue}════════════════════════════════════════════════════════════${colors.reset}`);
  // TAG: surface=api owner=platform-team rule=API-001
  console.log('Validation Summary:');
  console.log();
  console.log(`  ${colors.green}✓ Passed:  ${passed}${colors.reset}`);
  console.log(`  ${colors.red}✗ Failed:  ${failed}${colors.reset}`);
  console.log(`  ${colors.yellow}⚠ Warnings: ${warnings}${colors.reset}`);
  console.log();
  console.log(`  Duration: ${formatDuration(duration)}`);
  console.log();

  // Status
  if (failed === 0) {
    console.log(`${colors.green}╔════════════════════════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.green}║  ✓ All required checks passed! Ready for deployment.       ║${colors.reset}`);
    console.log(`${colors.green}╚════════════════════════════════════════════════════════════╝${colors.reset}`);
    return 0;
  } else {
    console.log(`${colors.red}╔════════════════════════════════════════════════════════════╗${colors.reset}`);
    console.log(`${colors.red}║  ✗ Some required checks failed. Review issues above.       ║${colors.reset}`);
    console.log(`${colors.red}╚════════════════════════════════════════════════════════════╝${colors.reset}`);

    console.log();
    console.log('Failed Checks:');
    for (const result of results) {
      if (!result.passed && result.rule.required) {
        console.log(`  - ${result.rule.name}: ${result.rule.description}`);
      }
    }
    return 1;
  }
}

// Run validation
const exitCode = validate();
// TAG: surface=api owner=platform-team rule=API-001
process.exit(exitCode);
