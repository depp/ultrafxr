using System;
using System.IO;
using System.Text;

namespace UltraFXR
{
	/// <summary>
	/// Writer for creating WAVE audio files.
	/// </summary>
	public class WaveWriter : IDisposable
	{
		private readonly BinaryWriter writer;
		private readonly bool isFloat;
		private readonly uint channelCount;
		private readonly uint sampleRate;
		private readonly uint sampleSize;
		private uint length;

		public WaveWriter(Stream stream, int sampleRate)
		{
			this.writer = new BinaryWriter(stream, Encoding.ASCII);
			this.isFloat = true;
			this.channelCount = 1;
			this.sampleRate = (uint)sampleRate;
			this.sampleSize = 4;
			this.writer.Write(new byte[44]);
		}

		/// <summary>
		/// Close this writer.
		/// </summary>
		public void Close()
		{
			this.WriteHeader();
			this.writer.Close();
		}

		/// <summary>
		/// Dispose of resources used by this object.
		/// </summary>
		public void Dispose()
		{
			this.Close();
			this.writer.Dispose();
		}

		/// <summary>
		/// Write audio data to the file.
		/// </summary>
		/// <param name="buffer">The audio data to write.</param>
		public void Write(float[] buffer)
		{
			foreach (float sample in buffer)
			{
				this.writer.Write(sample);
			}
			this.length += (uint)buffer.Length * 4;
		}

		/// <summary>
		/// Write the WAVE header.  This is always 44 bytes long.
		/// </summary>
		private void WriteHeader()
		{
			uint fsize = this.channelCount * this.sampleSize;

			this.writer.Seek(0, SeekOrigin.Begin);
			this.writer.Write(new char[] { 'R', 'I', 'F', 'F' });
			this.writer.Write(this.length + 36);
			this.writer.Write(new char[] { 'W', 'A', 'V', 'E' });
			this.writer.Write(new char[] { 'f', 'm', 't', ' ' });
			this.writer.Write((uint)16);
			this.writer.Write((ushort)(this.isFloat ? 3 : 1));
			this.writer.Write((ushort)this.channelCount);
			this.writer.Write((uint)this.sampleRate);
			this.writer.Write(this.sampleRate * fsize);
			this.writer.Write((ushort)(fsize));
			this.writer.Write((ushort)(this.sampleSize * 8));
			this.writer.Write(new char[] { 'd', 'a', 't', 'a' });
			this.writer.Write(this.length);
		}
	}
}
