/**
 * Performance Guard - Chatman Constant Enforcement
 * Ensures operations complete within ≤8 ticks
 * Aligned with DOCTRINE Chatman Constant principle
 */

interface PerformanceMetrics {
  operationId: string
  startTime: number
  endTime?: number
  duration?: number
  ticks: number
  violations: number
  isValid: boolean
}

const CHATMAN_CONSTANT = 8 // Maximum allowed ticks
const TICK_UNIT = 1000 // 1000ms = 1 tick

class PerformanceGuard {
  private metrics: Map<string, PerformanceMetrics> = new Map()
  private activeOperations: Map<string, number> = new Map()

  /**
   * Start measuring an operation
   */
  start(operationId: string): void {
    this.activeOperations.set(operationId, performance.now())
  }

  /**
   * End measuring and validate against Chatman Constant
   */
  end(operationId: string): PerformanceMetrics {
    const startTime = this.activeOperations.get(operationId)
    if (!startTime) {
      throw new Error(`Operation ${operationId} was never started`)
    }

    const endTime = performance.now()
    const duration = endTime - startTime
    const ticks = Math.ceil(duration / TICK_UNIT)
    const isValid = ticks <= CHATMAN_CONSTANT

    const metrics: PerformanceMetrics = {
      operationId,
      startTime,
      endTime,
      duration,
      ticks,
      violations: isValid ? 0 : 1,
      isValid,
    }

    if (!isValid) {
      console.warn(
        `⚠️ CHATMAN VIOLATION: Operation "${operationId}" took ${ticks} ticks (limit: ${CHATMAN_CONSTANT}). Duration: ${duration.toFixed(2)}ms`
      )
    }

    this.metrics.set(operationId, metrics)
    this.activeOperations.delete(operationId)

    return metrics
  }

  /**
   * Get metrics for an operation
   */
  getMetrics(operationId: string): PerformanceMetrics | undefined {
    return this.metrics.get(operationId)
  }

  /**
   * Get all metrics
   */
  getAllMetrics(): PerformanceMetrics[] {
    return Array.from(this.metrics.values())
  }

  /**
   * Get compliance statistics
   */
  getComplianceStats(): {
    totalOperations: number
    compliantOperations: number
    violatingOperations: number
    compliancePercentage: number
    averageTicks: number
  } {
    const allMetrics = Array.from(this.metrics.values())
    const compliant = allMetrics.filter((m) => m.isValid).length
    const violating = allMetrics.length - compliant

    return {
      totalOperations: allMetrics.length,
      compliantOperations: compliant,
      violatingOperations: violating,
      compliancePercentage:
        allMetrics.length > 0
          ? (compliant / allMetrics.length) * 100
          : 0,
      averageTicks:
        allMetrics.length > 0
          ? allMetrics.reduce((sum, m) => sum + m.ticks, 0) / allMetrics.length
          : 0,
    }
  }

  /**
   * Async operation wrapper with auto-measurement
   */
  async measureAsync<T>(
    operationId: string,
    operation: () => Promise<T>
  ): Promise<T> {
    this.start(operationId)
    try {
      const result = await operation()
      this.end(operationId)
      return result
    } catch (err) {
      this.end(operationId)
      throw err
    }
  }

  /**
   * Sync operation wrapper with auto-measurement
   */
  measure<T>(operationId: string, operation: () => T): T {
    this.start(operationId)
    try {
      const result = operation()
      this.end(operationId)
      return result
    } catch (err) {
      this.end(operationId)
      throw err
    }
  }

  /**
   * Enforce Chatman Constant with strict mode
   */
  enforceStrict(metrics: PerformanceMetrics): void {
    if (!metrics.isValid) {
      throw new Error(
        `DOCTRINE VIOLATION: Operation "${metrics.operationId}" exceeded Chatman Constant (${metrics.ticks} > ${CHATMAN_CONSTANT} ticks)`
      )
    }
  }

  /**
   * Reset metrics
   */
  reset(): void {
    this.metrics.clear()
    this.activeOperations.clear()
  }
}

// Export singleton instance
export const performanceGuard = new PerformanceGuard()

export default PerformanceGuard
