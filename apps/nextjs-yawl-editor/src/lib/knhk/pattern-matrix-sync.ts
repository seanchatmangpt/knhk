/**
 * DOCTRINE ALIGNMENT: Q (Hard Invariants)
 * Pattern permutation matrix synchronization and validation
 *
 * COVENANT 2: Invariants Are Law
 * Pattern combinations MUST conform to the permutation matrix
 * Invalid patterns are REJECTED, not warned
 */

import { withSpan, createSpan } from '@/lib/telemetry/setup';
import { knhkConfig } from './config';
import type {
  PatternPermutations,
  PatternCombination,
  SplitType,
  JoinType,
  Modifiers,
  CacheInfo,
  Unsubscribe,
} from './types';

/* ============================================================================
 * Pattern Matrix Synchronization
 * ========================================================================== */

/**
 * Load and cache YAWL pattern permutation matrix
 *
 * The matrix defines all valid split-join-modifier combinations
 * This is the source of truth for pattern validation
 */
export class PatternMatrixSync {
  private matrix: PatternPermutations | null = null;
  private loadedAt: number = 0;
  private cacheHits: number = 0;
  private cacheMisses: number = 0;
  private updateCallbacks: Set<() => void> = new Set();
  private pollInterval: NodeJS.Timeout | null = null;

  constructor() {
    // Start polling for updates if enabled
    if (knhkConfig.patternMatrix.pollInterval > 0) {
      this.startPolling();
    }
  }

  /* ============================
   * Matrix Loading
   * ============================ */

  /**
   * Load pattern permutation matrix
   *
   * PERFORMANCE: Must complete within matrixLoadBudget (500ms)
   */
  async loadMatrix(): Promise<PatternPermutations> {
    const startTime = performance.now();

    // Check cache TTL
    if (this.matrix && this.isCacheValid()) {
      this.cacheHits++;
      return this.matrix;
    }

    this.cacheMisses++;

    return withSpan(
      'pattern.matrix.load',
      async () => {
        try {
          // Try loading from local file first
          const matrix = await this.loadFromLocal();
          this.matrix = matrix;
          this.loadedAt = Date.now();

          const duration = performance.now() - startTime;

          // Check performance budget
          if (duration > knhkConfig.performance.matrixLoadBudget) {
            console.warn(
              `Matrix load exceeded budget: ${duration}ms > ${knhkConfig.performance.matrixLoadBudget}ms`
            );
          }

          // Notify listeners
          this.notifyUpdate();

          return matrix;
        } catch (error) {
          console.error('Failed to load from local, trying kernel:', error);

          // Fallback to kernel if local load fails
          try {
            const matrix = await this.loadFromKernel();
            this.matrix = matrix;
            this.loadedAt = Date.now();
            this.notifyUpdate();
            return matrix;
          } catch (kernelError) {
            console.error('Failed to load from kernel:', kernelError);

            // Final fallback to bundled matrix
            const matrix = this.getBundledMatrix();
            this.matrix = matrix;
            this.loadedAt = Date.now();
            this.notifyUpdate();
            return matrix;
          }
        }
      },
      {
        'matrix.cache_hit': this.matrix !== null && this.isCacheValid(),
      }
    );
  }

  /**
   * Load matrix from local file (Turtle format)
   */
  private async loadFromLocal(): Promise<PatternPermutations> {
    const response = await fetch(knhkConfig.patternMatrix.url);

    if (!response.ok) {
      throw new Error(`Failed to fetch pattern matrix: ${response.statusText}`);
    }

    const turtleContent = await response.text();

    // Parse Turtle to extract pattern combinations
    // This is a simplified parser - in production, use a proper Turtle parser
    return this.parseTurtle(turtleContent);
  }

  /**
   * Load matrix from knhk kernel as fallback
   */
  private async loadFromKernel(): Promise<PatternPermutations> {
    const { knhkClient } = await import('./client');
    return knhkClient.getPatternMatrix();
  }

  /**
   * Get bundled fallback matrix
   */
  private getBundledMatrix(): PatternPermutations {
    // Hardcoded fallback with common valid patterns
    return {
      version: '1.0.0-fallback',
      source: 'bundled',
      lastUpdated: new Date().toISOString(),
      combinations: [
        // AND split + AND join (parallel split + synchronization)
        { split: 'AND', join: 'AND', modifiers: [], valid: true, constraints: undefined, examples: undefined },

        // XOR split + XOR join (exclusive choice + simple merge)
        { split: 'XOR', join: 'XOR', modifiers: [], valid: true, constraints: undefined, examples: undefined },

        // OR split + OR join (multi-choice + multi-merge)
        { split: 'OR', join: 'OR', modifiers: [], valid: true, constraints: undefined, examples: undefined },

        // XOR split + AND join (discriminator pattern)
        { split: 'XOR', join: 'AND', modifiers: [], valid: true, constraints: undefined, examples: undefined },

        // Cancellation patterns
        { split: 'AND', join: 'AND', modifiers: ['cancel_region'], valid: true, constraints: undefined, examples: undefined },
        { split: 'XOR', join: 'XOR', modifiers: ['cancel_case'], valid: true, constraints: undefined, examples: undefined },

        // Multiple instance patterns
        { split: 'AND', join: 'AND', modifiers: ['multiple_instances'], valid: true, constraints: undefined, examples: undefined },

        // Deferred choice
        { split: 'XOR', join: 'XOR', modifiers: ['deferred_choice'], valid: true, constraints: undefined, examples: undefined },
      ],
      index: {},
    };
  }

  /**
   * Parse Turtle content to extract pattern combinations
   */
  private parseTurtle(content: string): PatternPermutations {
    const combinations: PatternCombination[] = [];

    // Split by triple lines (simplified parsing)
    const lines = content.split('\n').filter((line) => {
      const trimmed = line.trim();
      return trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('@');
    });

    // Look for pattern combination definitions
    // Format: :Pattern_AND_AND a yawl:ValidPattern ; yawl:split "AND" ; yawl:join "AND" .
    let currentPattern: Partial<PatternCombination> | null = null;

    for (const line of lines) {
      // New pattern definition
      if (line.includes('a yawl:ValidPattern') || line.includes('a yawl:InvalidPattern')) {
        if (currentPattern && currentPattern.split && currentPattern.join) {
          combinations.push({
            split: currentPattern.split,
            join: currentPattern.join,
            modifiers: currentPattern.modifiers || [],
            valid: currentPattern.valid ?? false,
            constraints: currentPattern.constraints ?? undefined,
            examples: currentPattern.examples ?? undefined,
          });
        }

        currentPattern = {
          valid: line.includes('ValidPattern'),
          modifiers: [],
        };
      }

      // Extract split type
      if (line.includes('yawl:split')) {
        const match = line.match(/yawl:split\s+"([^"]+)"/);
        if (match && currentPattern) {
          currentPattern.split = match[1] as SplitType;
        }
      }

      // Extract join type
      if (line.includes('yawl:join')) {
        const match = line.match(/yawl:join\s+"([^"]+)"/);
        if (match && currentPattern) {
          currentPattern.join = match[1] as JoinType;
        }
      }

      // Extract modifiers
      if (line.includes('yawl:modifier')) {
        const match = line.match(/yawl:modifier\s+"([^"]+)"/);
        if (match && currentPattern) {
          currentPattern.modifiers = currentPattern.modifiers || [];
          currentPattern.modifiers.push(match[1] as Modifiers);
        }
      }
    }

    // Add last pattern
    if (currentPattern && currentPattern.split && currentPattern.join) {
      combinations.push({
        split: currentPattern.split,
        join: currentPattern.join,
        modifiers: currentPattern.modifiers || [],
        valid: currentPattern.valid ?? false,
        constraints: currentPattern.constraints ?? undefined,
        examples: currentPattern.examples ?? undefined,
      });
    }

    // Build index
    const index: { [key: string]: PatternCombination } = {};
    for (const combo of combinations) {
      const key = this.getCombinationKey(combo.split, combo.join, combo.modifiers);
      index[key] = combo;
    }

    return {
      version: '1.0.0',
      source: knhkConfig.patternMatrix.url,
      lastUpdated: new Date().toISOString(),
      combinations,
      index,
    };
  }

  /* ============================
   * Validation
   * ============================ */

  /**
   * Validate a split-join-modifier combination
   *
   * CRITICAL: Hot path operation, must complete in ≤8 ticks (Chatman Constant)
   *
   * @returns true if combination is valid, false otherwise
   */
  validateCombination(split: SplitType, join: JoinType, modifiers: Modifiers[] = []): boolean {
    const startTime = performance.now();

    if (!this.matrix) {
      console.warn('Pattern matrix not loaded, cannot validate');
      return false; // Fail closed - reject if matrix unavailable
    }

    const key = this.getCombinationKey(split, join, modifiers);
    const combination = this.matrix.index[key];

    const duration = performance.now() - startTime;

    // Check hot path budget (8 ticks = ~8ms)
    if (duration > knhkConfig.performance.hotPathBudget) {
      console.warn(
        `Pattern validation exceeded hot path budget: ${duration}ms > ${knhkConfig.performance.hotPathBudget}ms`
      );
    }

    // Return validity (default to false if not found)
    return combination?.valid ?? false;
  }

  /**
   * Get all valid join types for a given split type
   */
  getValidJoinsFor(split: SplitType): JoinType[] {
    if (!this.matrix) {
      return [];
    }

    const validJoins = new Set<JoinType>();

    for (const combo of this.matrix.combinations) {
      if (combo.split === split && combo.valid) {
        validJoins.add(combo.join);
      }
    }

    return Array.from(validJoins);
  }

  /**
   * Get all valid split types for a given join type
   */
  getValidSplitsFor(join: JoinType): SplitType[] {
    if (!this.matrix) {
      return [];
    }

    const validSplits = new Set<SplitType>();

    for (const combo of this.matrix.combinations) {
      if (combo.join === join && combo.valid) {
        validSplits.add(combo.split);
      }
    }

    return Array.from(validSplits);
  }

  /**
   * Get all valid modifiers for a split-join combination
   */
  getValidModifiersFor(split: SplitType, join: JoinType): Modifiers[] {
    if (!this.matrix) {
      return [];
    }

    const validModifiers = new Set<Modifiers>();

    for (const combo of this.matrix.combinations) {
      if (combo.split === split && combo.join === join && combo.valid) {
        combo.modifiers.forEach((mod) => validModifiers.add(mod));
      }
    }

    return Array.from(validModifiers);
  }

  /* ============================
   * Cache Management
   * ============================ */

  /**
   * Check if cache is still valid
   */
  private isCacheValid(): boolean {
    if (!this.matrix || this.loadedAt === 0) {
      return false;
    }

    const age = (Date.now() - this.loadedAt) / 1000; // seconds
    return age < knhkConfig.patternMatrix.cacheTTL;
  }

  /**
   * Get cache status
   */
  getCacheStatus(): CacheInfo {
    return {
      loaded: this.matrix !== null,
      version: this.matrix?.version ?? undefined,
      lastUpdated: this.matrix?.lastUpdated ?? undefined,
      size: this.matrix?.combinations.length || 0,
      hits: this.cacheHits,
      misses: this.cacheMisses,
    };
  }

  /**
   * Clear cache and force reload
   */
  clearCache(): void {
    this.matrix = null;
    this.loadedAt = 0;
  }

  /* ============================
   * Update Notifications
   * ============================ */

  /**
   * Register callback for matrix updates
   */
  onMatrixUpdated(callback: () => void): Unsubscribe {
    this.updateCallbacks.add(callback);

    return () => {
      this.updateCallbacks.delete(callback);
    };
  }

  /**
   * Notify all listeners of matrix update
   */
  private notifyUpdate(): void {
    this.updateCallbacks.forEach((callback) => {
      try {
        callback();
      } catch (error) {
        console.error('Error in matrix update callback:', error);
      }
    });
  }

  /* ============================
   * Polling
   * ============================ */

  /**
   * Start polling for matrix updates
   */
  private startPolling(): void {
    if (this.pollInterval) {
      return;
    }

    this.pollInterval = setInterval(() => {
      this.checkForUpdates().catch((error) => {
        console.error('Failed to check for matrix updates:', error);
      });
    }, knhkConfig.patternMatrix.pollInterval);
  }

  /**
   * Stop polling for updates
   */
  stopPolling(): void {
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }
  }

  /**
   * Check for matrix updates
   */
  private async checkForUpdates(): Promise<void> {
    if (!this.matrix) {
      return;
    }

    const span = createSpan('pattern.matrix.check_updates');

    try {
      // Fetch current version
      const response = await fetch(knhkConfig.patternMatrix.url, {
        method: 'HEAD',
      });

      if (!response.ok) {
        span.end();
        return;
      }

      const lastModified = response.headers.get('Last-Modified');
      const currentLastUpdated = this.matrix.lastUpdated;

      // If modified, reload
      if (lastModified && new Date(lastModified).getTime() > new Date(currentLastUpdated).getTime()) {
        await this.loadMatrix();
      }
    } catch (error) {
      console.error('Error checking for updates:', error);
    } finally {
      span.end();
    }
  }

  /* ============================
   * Utilities
   * ============================ */

  /**
   * Generate combination key for indexing
   */
  private getCombinationKey(split: SplitType, join: JoinType, modifiers: Modifiers[]): string {
    const sortedMods = modifiers.slice().sort();
    return `${split}:${join}:${sortedMods.join(',')}`;
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    this.stopPolling();
    this.updateCallbacks.clear();
    this.matrix = null;
  }
}

/* ============================================================================
 * Singleton Instance
 * ========================================================================== */

/**
 * Default pattern matrix sync instance
 */
export const patternMatrixSync = new PatternMatrixSync();

// Preload matrix on module import
patternMatrixSync.loadMatrix().catch((error) => {
  console.error('Failed to preload pattern matrix:', error);
});
