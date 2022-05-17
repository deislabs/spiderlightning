pub mod kv {
  #[allow(unused_imports)]
  use wit_bindgen_wasmtime::{wasmtime, anyhow};
  #[repr(u8)]
  #[derive(Clone, Copy, PartialEq, Eq)]
  pub enum Error {
    Success,
    Error,
  }
  impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        Error::Success => {
          f.debug_tuple("Error::Success").finish()
        }
        Error::Error => {
          f.debug_tuple("Error::Error").finish()
        }
      }
    }
  }
  pub type PayloadParam<'a,> = &'a [u8];
  pub type PayloadResult = Vec<u8>;
  pub trait Kv: Sized {
    type KvRd: std::fmt::Debug;
    fn get_kv(&mut self,) -> Result<Self::KvRd,Error>;
    
    fn get(&mut self,key: & str,rd: & Self::KvRd,) -> Result<PayloadResult,Error>;
    
    fn set(&mut self,key: & str,value: PayloadParam<'_,>,rd: & Self::KvRd,) -> Result<(),Error>;
    
    fn drop_kv_rd(&mut self, state: Self::KvRd) {
      drop(state);
    }
  }
  
  pub struct KvTables<T: Kv> {
    pub(crate) kv_rd_table: wit_bindgen_wasmtime::Table<T::KvRd>,
  }
  impl<T: Kv> Default for KvTables<T> {
    fn default() -> Self { Self {kv_rd_table: Default::default(),}}}
    pub fn add_to_linker<T, U>(linker: &mut wasmtime::Linker<T>, get: impl Fn(&mut T) -> (&mut U, &mut KvTables<U>)+ Send + Sync + Copy + 'static) -> anyhow::Result<()> 
    where U: Kv
    {
      use wit_bindgen_wasmtime::rt::get_memory;
      use wit_bindgen_wasmtime::rt::get_func;
      linker.func_wrap("kv", "get-kv", move |mut caller: wasmtime::Caller<'_, T>,arg0:i32| {
        let memory = &get_memory(&mut caller, "memory")?;
        let host = get(caller.data_mut());
        let (host, _tables) = host;
        let result = host.get_kv();
        match result {
          Ok(e) => { {
            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
            let (_, _tables) = get(data);
            caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
            caller_memory.store(arg0 + 4, wit_bindgen_wasmtime::rt::as_i32(_tables.kv_rd_table.insert(e) as i32))?;
          } },
          Err(e) => { {
            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
            let (_, _tables) = get(data);
            caller_memory.store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
            caller_memory.store(arg0 + 4, wit_bindgen_wasmtime::rt::as_i32(e as i32) as u8)?;
          } },
        };Ok(())
      })?;
      linker.func_wrap("kv", "get", move |mut caller: wasmtime::Caller<'_, T>,arg0:i32,arg1:i32,arg2:i32,arg3:i32| {
        
        let func = get_func(&mut caller, "canonical_abi_realloc")?;
        let func_canonical_abi_realloc = func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
        let memory = &get_memory(&mut caller, "memory")?;
        let (mem, data) = memory.data_and_store_mut(&mut caller);
        let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
        let host = get(data);
        let (host, _tables) = host;
        let ptr0 = arg0;
        let len0 = arg1;
        let param0 = _bc.slice_str(ptr0, len0)?;
        let param1 = _tables.kv_rd_table.get((arg2) as u32).ok_or_else(|| {
          wasmtime::Trap::new("invalid handle index")
        })?;
        let result = host.get(param0, param1, );
        match result {
          Ok(e) => { {
            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
            let (_, _tables) = get(data);
            caller_memory.store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
            let vec1 = e;
            let ptr1 = func_canonical_abi_realloc.call(&mut caller, (0, 0, 1, (vec1.len() as i32) * 1))?;
            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
            let (_, _tables) = get(data);
            caller_memory.store_many(ptr1, &vec1)?;
            caller_memory.store(arg3 + 8, wit_bindgen_wasmtime::rt::as_i32(vec1.len() as i32))?;
            caller_memory.store(arg3 + 4, wit_bindgen_wasmtime::rt::as_i32(ptr1))?;
          } },
          Err(e) => { {
            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
            let (_, _tables) = get(data);
            caller_memory.store(arg3 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
            caller_memory.store(arg3 + 4, wit_bindgen_wasmtime::rt::as_i32(e as i32) as u8)?;
          } },
        };Ok(())
      })?;
      linker.func_wrap("kv", "set", move |mut caller: wasmtime::Caller<'_, T>,arg0:i32,arg1:i32,arg2:i32,arg3:i32,arg4:i32,arg5:i32| {
        let memory = &get_memory(&mut caller, "memory")?;
        let (mem, data) = memory.data_and_store_mut(&mut caller);
        let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
        let host = get(data);
        let (host, _tables) = host;
        let ptr0 = arg0;
        let len0 = arg1;
        let ptr1 = arg2;
        let len1 = arg3;
        let param0 = _bc.slice_str(ptr0, len0)?;
        let param1 = _bc.slice(ptr1, len1)?;
        let param2 = _tables.kv_rd_table.get((arg4) as u32).ok_or_else(|| {
          wasmtime::Trap::new("invalid handle index")
        })?;
        let result = host.set(param0, param1, param2, );
        match result {
          Ok(e) => { {
            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
            let (_, _tables) = get(data);
            caller_memory.store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
            let () = e;
          } },
          Err(e) => { {
            let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
            let (_, _tables) = get(data);
            caller_memory.store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
            caller_memory.store(arg5 + 1, wit_bindgen_wasmtime::rt::as_i32(e as i32) as u8)?;
          } },
        };Ok(())
      })?;
      linker.func_wrap(
      "canonical_abi",
      "resource_drop_kv-rd",
      move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
        let (host, tables) = get(caller.data_mut());
        let handle = tables
        .kv_rd_table
        .remove(handle)
        .map_err(|e| {
          wasmtime::Trap::new(format!("failed to remove handle: {}", e))
        })?;
        host.drop_kv_rd(handle);
        Ok(())
      }
      )?;
      Ok(())
    }
    use wit_bindgen_wasmtime::rt::RawMem;
  }
  