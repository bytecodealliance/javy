use std::{
    collections::BinaryHeap,
    sync::{Arc, Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    hold, hold_and_release,
    quickjs::{prelude::MutFn, Ctx, Function, Value},
    to_js_error, val_to_string, Args,
};
use anyhow::{anyhow, Result};

/// Timer entry in the timer queue
#[derive(Debug, Clone)]
struct Timer {
    id: u32,
    fire_time: u64, // milliseconds since UNIX epoch
    callback: String, // JavaScript code to execute
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.fire_time == other.fire_time
    }
}

impl Eq for Timer {}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for min-heap behavior
        other.fire_time.cmp(&self.fire_time)
    }
}

/// Global timer queue
#[derive(Debug)]
struct TimerQueue {
    timers: BinaryHeap<Timer>,
    next_id: u32,
}

impl TimerQueue {
    fn new() -> Self {
        Self {
            timers: BinaryHeap::new(),
            next_id: 1,
        }
    }

    fn add_timer(&mut self, delay_ms: u32, callback: String) -> u32 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let id = self.next_id;
        self.next_id += 1;
        
        let timer = Timer {
            id,
            // For delay=0, set fire_time to current time to ensure immediate availability
            fire_time: if delay_ms == 0 { now } else { now + delay_ms as u64 },
            callback,
        };
        
        self.timers.push(timer);
        id
    }

    fn remove_timer(&mut self, timer_id: u32) -> bool {
        let original_len = self.timers.len();
        self.timers.retain(|timer| timer.id != timer_id);
        self.timers.len() != original_len
    }

    fn get_expired_timers(&mut self) -> Vec<Timer> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let mut expired = Vec::new();
        
        while let Some(timer) = self.timers.peek() {
            if timer.fire_time <= now {
                expired.push(self.timers.pop().unwrap());
            } else {
                break;
            }
        }
        
        expired
    }

    fn has_pending_timers(&self) -> bool {
        !self.timers.is_empty()
    }
}

static TIMER_QUEUE: OnceLock<Arc<Mutex<TimerQueue>>> = OnceLock::new();

fn get_timer_queue() -> &'static Arc<Mutex<TimerQueue>> {
    TIMER_QUEUE.get_or_init(|| Arc::new(Mutex::new(TimerQueue::new())))
}

/// Register timer functions on the global object
pub(crate) fn register(this: Ctx<'_>) -> Result<()> {
    let globals = this.globals();

    globals.set(
        "setTimeout",
        Function::new(
            this.clone(),
            MutFn::new(move |cx, args| {
                let (cx, args) = hold_and_release!(cx, args);
                set_timeout(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
            }),
        )?,
    )?;

    globals.set(
        "clearTimeout",
        Function::new(
            this.clone(),
            MutFn::new(move |cx, args| {
                let (cx, args) = hold_and_release!(cx, args);
                clear_timeout(hold!(cx.clone(), args)).map_err(|e| to_js_error(cx, e))
            }),
        )?,
    )?;

    Ok(())
}

fn set_timeout<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (ctx, args) = args.release();
    let args = args.into_inner();
    
    if args.is_empty() {
        return Err(anyhow!("setTimeout requires at least 1 argument"));
    }

    // Get callback (can be function or string)
    let callback_str = if args[0].is_function() {
        // Convert function to string representation
        val_to_string(&ctx, args[0].clone())?
    } else {
        // Treat as string code
        val_to_string(&ctx, args[0].clone())?
    };

    // Get delay (default to 0 if not provided)
    let delay_ms = if args.len() > 1 {
        args[1].as_number().unwrap_or(0.0).max(0.0) as u32
    } else {
        0
    };

    let mut queue = get_timer_queue().lock().unwrap();
    let timer_id = queue.add_timer(delay_ms, callback_str);
    
    Ok(Value::new_int(ctx, timer_id as i32))
}

fn clear_timeout<'js>(args: Args<'js>) -> Result<Value<'js>> {
    let (ctx, args) = args.release();
    let args = args.into_inner();
    
    if args.is_empty() {
        return Ok(Value::new_undefined(ctx));
    }

    let timer_id = args[0].as_number().unwrap_or(0.0) as u32;
    
    let mut queue = get_timer_queue().lock().unwrap();
    queue.remove_timer(timer_id);
    
    Ok(Value::new_undefined(ctx))
}

/// Process expired timers - should be called by the event loop
pub fn process_timers(ctx: Ctx<'_>) -> Result<()> {
    let mut queue = get_timer_queue().lock().unwrap();
    let expired_timers = queue.get_expired_timers();
    drop(queue); // Release lock before executing JavaScript
    
    for timer in expired_timers {
        // Execute the timer callback
        if let Err(e) = ctx.eval::<(), _>(timer.callback.as_str()) {
            eprintln!("Timer callback error: {}", e);
        }
    }
    
    Ok(())
}

/// Check if there are pending timers
pub fn has_pending_timers() -> bool {
    let queue = get_timer_queue().lock().unwrap();
    queue.has_pending_timers()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Config, Runtime};
    use anyhow::Error;

    #[test]
    fn test_timer_queue() {
        let mut queue = TimerQueue::new();
        
        // Add some timers
        let id1 = queue.add_timer(100, "console.log('timer1')".to_string());
        let id2 = queue.add_timer(50, "console.log('timer2')".to_string());
        let id3 = queue.add_timer(200, "console.log('timer3')".to_string());
        
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        
        assert!(queue.has_pending_timers());
        
        // Remove a timer
        assert!(queue.remove_timer(id2));
        assert!(!queue.remove_timer(999)); // Non-existent timer
        
        assert!(queue.has_pending_timers());
    }

    #[test]
    fn test_register() -> Result<()> {
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Check that setTimeout is available
            let result: Value = cx.eval("typeof setTimeout")?;
            let type_str = val_to_string(&cx, result)?;
            assert_eq!(type_str, "function");
            
            // Check that clearTimeout is available
            let result: Value = cx.eval("typeof clearTimeout")?;
            let type_str = val_to_string(&cx, result)?;
            assert_eq!(type_str, "function");
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_set_timeout_basic() -> Result<()> {
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Test setTimeout with string callback
            let result: Value = cx.eval("setTimeout('1+1', 100)")?;
            let timer_id = result.as_number().unwrap() as i32;
            assert!(timer_id > 0);
            
            // Test setTimeout with function callback  
            let result: Value = cx.eval("setTimeout(function() { return 42; }, 50)")?;
            let timer_id2 = result.as_number().unwrap() as i32;
            assert!(timer_id2 > timer_id);
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_clear_timeout() -> Result<()> {
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Create a timer and clear it
            let result: Value = cx.eval("const id = setTimeout('console.log(\"test\")', 1000); clearTimeout(id); id")?;
            let timer_id = result.as_number().unwrap() as i32;
            assert!(timer_id > 0);
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_timer_execution() -> Result<()> {
        // Clear any existing timers from other tests
        {
            let mut queue = get_timer_queue().lock().unwrap();
            queue.timers.clear();
        }
        
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Use a unique variable name to avoid interference between tests
            let unique_var = format!("timerExecuted_{}", std::process::id());
            let timer_code = format!("globalThis.{} = false; setTimeout('globalThis.{} = true', 0)", unique_var, unique_var);
            cx.eval::<(), _>(timer_code.as_str())?;
            
            // Process timers immediately without sleep - they should be available
            process_timers(cx.clone())?;
            
            // Check if timer was executed
            let check_code = format!("globalThis.{}", unique_var);
            let result: Value = cx.eval(check_code.as_str())?;
            
            assert!(result.as_bool().unwrap_or(false));
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_timer_with_delay() -> Result<()> {
        // Clear any existing timers from other tests
        {
            let mut queue = get_timer_queue().lock().unwrap();
            queue.timers.clear();
        }
        
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Use unique variable name to avoid interference between tests
            let unique_var = format!("delayedTimer_{}", std::process::id());
            
            // Set a timer with a delay that shouldn't fire immediately
            let timer_code = format!("globalThis.{} = false; setTimeout('globalThis.{} = true', 1000)", unique_var, unique_var);
            cx.eval::<(), _>(timer_code.as_str())?;
            
            // Process timers immediately - should not execute
            process_timers(cx.clone())?;
            
            // Check if timer was NOT executed
            let check_code = format!("globalThis.{}", unique_var);
            let result: Value = cx.eval(check_code.as_str())?;
            assert!(!result.as_bool().unwrap_or(true));
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_multiple_timers() -> Result<()> {
        // Clear any existing timers from other tests
        {
            let mut queue = get_timer_queue().lock().unwrap();
            queue.timers.clear();
        }
        
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Use unique variable names to avoid interference between tests
            let unique_id = std::process::id();
            let timer1_var = format!("timer1_{}", unique_id);
            let timer2_var = format!("timer2_{}", unique_id);
            
            // Set multiple timers
            let timer_code = format!("
                globalThis.{} = false;
                globalThis.{} = false;
                setTimeout('globalThis.{} = true', 0);
                setTimeout('globalThis.{} = true', 0);
            ", timer1_var, timer2_var, timer1_var, timer2_var);
            cx.eval::<(), _>(timer_code.as_str())?;
            
            // Process timers
            process_timers(cx.clone())?;
            
            // Check if both timers were executed
            let check1_code = format!("globalThis.{}", timer1_var);
            let check2_code = format!("globalThis.{}", timer2_var);
            let result1: Value = cx.eval(check1_code.as_str())?;
            let result2: Value = cx.eval(check2_code.as_str())?;
            assert!(result1.as_bool().unwrap_or(false));
            assert!(result2.as_bool().unwrap_or(false));
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_clear_timeout_removes_timer() -> Result<()> {
        // Clear any existing timers from other tests
        {
            let mut queue = get_timer_queue().lock().unwrap();
            queue.timers.clear();
        }
        
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Use unique variable name to avoid interference between tests
            let unique_var = format!("clearedTimer_{}", std::process::id());
            
            // Set a timer and immediately clear it
            let timer_code = format!("
                globalThis.{} = false;
                const id = setTimeout('globalThis.{} = true', 0);
                clearTimeout(id);
            ", unique_var, unique_var);
            cx.eval::<(), _>(timer_code.as_str())?;
            
            // Process timers
            process_timers(cx.clone())?;
            
            // Check if timer was NOT executed
            let check_code = format!("globalThis.{}", unique_var);
            let result: Value = cx.eval(check_code.as_str())?;
            assert!(!result.as_bool().unwrap_or(true));
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }

    #[test]
    fn test_has_pending_timers() -> Result<()> {
        // Clear any existing timers from other tests and verify clean state
        {
            let mut queue = get_timer_queue().lock().unwrap();
            queue.timers.clear();
        }
        
        // Initially no pending timers
        assert!(!has_pending_timers());
        
        let mut config = Config::default();
        config.timers(true);
        let runtime = Runtime::new(config)?;
        runtime.context().with(|cx| {
            register(cx.clone())?;
            
            // Add a timer
            cx.eval::<(), _>("setTimeout('console.log(\"test\")', 1000)")?;
            
            // Should have pending timers
            assert!(has_pending_timers());
            
            Ok::<_, Error>(())
        })?;
        Ok(())
    }
} 