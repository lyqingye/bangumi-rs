use anyhow::Result;
use async_trait::async_trait;
use std::{collections::HashMap, hash::Hash, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    Start,
    Stop,
    Cancel,
    Finish,
    Fail,
    Retry,
    Fallback,
    Sync,
}

#[derive(Debug, Clone)]
pub struct Context {}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum State {
    Pending,
    Running,
    Stopped,
    Cancelled,
    Failed,
    Finished,
    Retrying,
}

// Guard trait - 检查状态转换是否可以进行
#[async_trait]
pub trait Guard<C, E, S>: Send + Sync {
    async fn check(&self, ctx: &C, event: &E, state: &S) -> Result<bool>;
}

// Action trait - 执行状态转换时的动作
#[async_trait]
pub trait Action<C, E, S>: Send + Sync {
    async fn execute(&self, ctx: &mut C, event: &E, state: &S) -> Result<(Option<E>, Option<S>)>;
}

// 默认的 Guard 实现 - 总是返回 true
pub struct AlwaysTrue;

#[async_trait]
impl<C, E, S> Guard<C, E, S> for AlwaysTrue
where
    C: Send + Sync,
    E: Send + Sync,
    S: Send + Sync,
{
    async fn check(&self, _ctx: &C, _event: &E, _state: &S) -> Result<bool> {
        Ok(true)
    }
}

// 默认的 Action 实现 - 什么都不做
pub struct NoOp;

#[async_trait]
impl<C, E, S> Action<C, E, S> for NoOp
where
    C: Send + Sync,
    E: Send + Sync,
    S: Send + Sync,
{
    async fn execute(
        &self,
        _ctx: &mut C,
        _event: &E,
        _state: &S,
    ) -> Result<(Option<E>, Option<S>)> {
        Ok((None, None))
    }
}

// 总是返回 true 的简单 Guard
pub struct TrueGuard;

#[async_trait]
impl<C, E, S> Guard<C, E, S> for TrueGuard
where
    C: Send + Sync,
    E: Send + Sync,
    S: Send + Sync,
{
    async fn check(&self, _ctx: &C, _event: &E, _state: &S) -> Result<bool> {
        Ok(true)
    }
}

// 简单的总是返回 None 的 Action
pub struct EmptyAction;

#[async_trait]
impl<C, E, S> Action<C, E, S> for EmptyAction
where
    C: Send + Sync,
    E: Send + Sync,
    S: Send + Sync,
{
    async fn execute(
        &self,
        _ctx: &mut C,
        _event: &E,
        _state: &S,
    ) -> Result<(Option<E>, Option<S>)> {
        Ok((None, None))
    }
}

// 状态转换定义
pub struct Transition<C, E, S> {
    pub from: S,
    pub to: S,
    pub guards: Vec<Arc<dyn Guard<C, E, S>>>,
    pub actions: Vec<Arc<dyn Action<C, E, S>>>,
}

// 状态转换表
pub struct TransitionTable<C, E, S: Hash + Eq + Clone> {
    transitions: HashMap<S, Transition<C, E, S>>,
}

impl<C, E, S> TransitionTable<C, E, S>
where
    C: Send + Sync + 'static,
    E: Send + Sync + 'static + Hash + Eq + Clone,
    S: Send + Sync + 'static + Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
        }
    }

    pub fn add(&mut self, transition: Transition<C, E, S>) {
        self.transitions.insert(transition.from.clone(), transition);
    }

    pub fn add_transition(
        &mut self,
        from: S,
        to: S,
        guard: Arc<dyn Guard<C, E, S>>,
        action: Arc<dyn Action<C, E, S>>,
    ) {
        let transition = Transition {
            from: from.clone(),
            to,
            guards: vec![guard],
            actions: vec![action],
        };
        self.add(transition);
    }

    pub async fn process(
        &self,
        ctx: &mut C,
        event: &E,
        current_state: &S,
    ) -> Result<Option<S>> {
        if let Some(transition) = self.transitions.get(current_state) {
            // 检查所有guard
            for guard in &transition.guards {
                if !guard.check(ctx, event, current_state).await? {
                    return Ok(None); // Guard失败，不执行转换
                }
            }

            // 执行所有action
            for action in &transition.actions {
                let (next_event, next_state) = action.execute(ctx, event, current_state).await?;

                // 处理可能的额外状态转换
                if let Some(_next_event) = next_event {
                    // TODO: 处理下一个事件
                }

                if let Some(state) = next_state {
                    return Ok(Some(state));
                }
            }

            return Ok(Some(transition.to.clone()));
        }

        Ok(None) // 没有找到匹配的转换
    }
}

// 状态机
pub struct StateMachine {
    table: TransitionTable<Context, Event, State>,
    context: Context,
    current_state: State,
}

impl StateMachine {
    pub fn new(table: TransitionTable<Context, Event, State>) -> Self {
        Self {
            table,
            context: Context {},
            current_state: State::Pending,
        }
    }

    pub async fn process_event(&mut self, event: Event) -> Result<()> {
        if let Some(new_state) = self
            .table
            .process(&mut self.context, &event, &self.current_state)
            .await?
        {
            self.current_state = new_state;
        }
        Ok(())
    }

    pub fn current_state(&self) -> &State {
        &self.current_state
    }
}

// 使用示例
pub fn build_transition_table() -> TransitionTable<Context, Event, State> {
    let mut table = TransitionTable::new();

    // 使用默认实现
    table.add_transition(
        State::Pending,
        State::Running,
        Arc::new(AlwaysTrue),
        Arc::new(NoOp),
    );

    // 使用函数转换
    table.add_transition(
        State::Running,
        State::Finished,
        Arc::new(TrueGuard),
        Arc::new(EmptyAction),
    );

    table
}

// 自定义 Action 示例
pub struct CompleteAction;

#[async_trait]
impl Action<Context, Event, State> for CompleteAction {
    async fn execute(
        &self,
        _ctx: &mut Context,
        _event: &Event,
        _state: &State,
    ) -> Result<(Option<Event>, Option<State>)> {
        // 可以在这里实现复杂逻辑
        println!("Task completed!");
        Ok((None, Some(State::Finished)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_machine() -> Result<()> {
        let table = build_transition_table();
        let mut machine = StateMachine::new(table);

        // 初始状态应该是Pending
        assert!(matches!(machine.current_state(), State::Pending));

        // 处理Start事件
        machine.process_event(Event::Start).await?;
        assert!(matches!(machine.current_state(), State::Running));

        // 处理Finish事件
        machine.process_event(Event::Finish).await?;
        assert!(matches!(machine.current_state(), State::Finished));

        Ok(())
    }
}
