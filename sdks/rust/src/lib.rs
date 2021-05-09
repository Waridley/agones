// Copyright 2018 Google LLC All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod proto {
	#[cfg(feature = "sdk")]
	pub mod sdk {
		tonic::include_proto!("agones.dev.sdk");
	}
	#[cfg(feature = "alpha")]
	pub mod alpha {
		tonic::include_proto!("agones.dev.sdk.alpha");
	}
	#[cfg(feature = "beta")]
	pub mod beta {
		tonic::include_proto!("agones.dev.sdk.beta");
	}
	#[cfg(feature = "allocation")]
	pub mod allocation {
		tonic::include_proto!("allocation");
	}
}

pub mod sdk {
	#[cfg(feature = "sdk")]
	pub use crate::proto::sdk::{
		Empty, KeyValue, Duration,
	};
	#[cfg(feature = "sdk")]
	use crate::proto::sdk::sdk_client::SdkClient;
	#[cfg(feature = "alpha")]
	use crate::proto::alpha;
	
	use tonic::{
		Response, Status,
		body::Body,
		codegen::{StdError, HttpBody, },
		transport::{Channel, Endpoint,}
	};
	use std::net::{Ipv6Addr, Ipv4Addr, SocketAddrV6, SocketAddrV4};
	use tonic::codegen::http::uri::{Builder as UriBuilder, Scheme};
	use tokio_stream::StreamExt;
	use std::ops::{Deref, DerefMut};
	
	#[cfg(feature = "sdk")]
	#[derive(Clone)]
	pub struct Sdk<T> {
		stable: SdkClient<T>,
		#[cfg(feature = "alpha")]
		alpha: alpha::sdk_client::SdkClient<T>,
	}
	
	impl<T> Deref for Sdk<T> {
		type Target = SdkClient<T>;
		
		fn deref(&self) -> &Self::Target {
			&self.stable
		}
	}
	
	impl<T> DerefMut for Sdk<T> {
		fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.stable
		}
	}
	
	#[cfg(all(feature = "sdk", feature = "alpha"))]
	impl<T> Sdk<T> {
		pub fn alpha(&mut self) -> &mut alpha::sdk_client::SdkClient<T> {
			&mut self.alpha
		}
	}
	
	#[cfg(all(feature = "sdk", feature = "transport"))]
	impl Sdk<Channel> {
		pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
		where
			D: std::convert::TryInto<tonic::transport::Endpoint>,
			D::Error: Into<StdError>,
		{
			let chan = Endpoint::new(dst)?.connect().await?;
			
			Ok(Self::new(chan))
		}
		
		pub fn connect_lazy<D>(dst: D) -> Result<Self, tonic::transport::Error>
		where
			D: std::convert::TryInto<tonic::transport::Endpoint>,
			D::Error: Into<StdError>,
		{
			let chan = Endpoint::new(dst)?.connect_lazy()?;
			
			Ok(Self::new(chan))
		}
		
		pub async fn new_v4(port: u16) -> Result<Self, tonic::transport::Error> {
			Self::connect(UriBuilder::new()
				.scheme(Scheme::HTTP)
				.authority(
					&*SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string()
				)
				.path_and_query("")
				.build()
				.unwrap()
			).await
		}
		
		pub async fn new_v6(port: u16) -> Result<Self, tonic::transport::Error> {
			Self::connect(UriBuilder::new()
				.scheme(Scheme::HTTP)
				.authority(
					&*SocketAddrV6::new(Ipv6Addr::LOCALHOST, port, 0, 0).to_string()
				)
				.path_and_query("")
				.build()
				.unwrap()
			).await
		}
		
		pub async fn default_v4() -> Result<Self, tonic::transport::Error> {
			Self::new_v4(9357).await
		}
		
		pub async fn default_v6() -> Result<Self, tonic::transport::Error> {
			Self::new_v6(9357).await
		}
		
		pub async fn health(&mut self) -> Result<Response<Empty>, Status> {
			use tokio::time::{interval, Duration};
			
			let dur = interval(Duration::from_secs(10));
			let stream = tokio_stream::wrappers::IntervalStream::new(dur)
				.map(|_| Empty {});
			self.stable.health(stream).await
		}
	}
	
	#[cfg(feature = "sdk")]
	impl<T> Sdk<T>
	where
		T: tonic::client::GrpcService<tonic::body::BoxBody>,
		T::ResponseBody: Body + HttpBody + Send + 'static,
		T::Error: Into<StdError>,
		<T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
	{
		#[cfg(not(feature = "alpha"))]
		pub fn new(inner: T) -> Self {
			Self {
				stable: SdkClient::new(inner),
			}
		}
		
		#[cfg(feature = "alpha")]
		pub fn new(inner: T) -> Self where T: Clone {
			Self {
				alpha: alpha::sdk_client::SdkClient::new(inner.clone()),
				stable: SdkClient::new(inner),
			}
		}
		
		#[cfg(feature = "alpha")]
		pub fn from_connectors(stable: T, alpha: T) -> Self {
			Self {
				stable: SdkClient::new(stable),
				alpha: alpha::sdk_client::SdkClient::new(alpha)
			}
		}
		
		/// Call when the GameServer is ready
		pub async fn ready(&mut self) -> Result<Response<Empty>, Status> {
			self.stable.ready(Empty {}).await
		}
		
		pub async fn shutdown(&mut self) -> Result<Response<Empty>, Status> {
			self.stable.shutdown(Empty {}).await
		}
		
		pub async fn set_label(
			&mut self,
			request: impl tonic::IntoRequest<KeyValue>,
		) -> Result<Response<Empty>, Status> {
			self.stable.set_label(request).await
		}
		
		pub async fn set_annotation(&mut self, key: impl ToString, value: impl ToString) -> Result<Response<Empty>, Status> {
			self.stable.set_annotation(KeyValue {
				key: key.to_string(),
				value: value.to_string()
			}).await
		}
		
		pub async fn allocate(&mut self) -> Result<Response<Empty>, Status> {
			self.stable.allocate(Empty {}).await
		}
		
		pub async fn reserve(&mut self, seconds: i64) -> Result<Response<Empty>, Status> {
			self.stable.reserve(Duration { seconds }).await
		}
	}
}