using System;
using System.IO;
using System.Text;
using Moria.UltraFXR;

namespace UltraFXR
{
	class MainClass
	{
		public static void Main(string[] args)
		{
			if (args.Length != 2)
			{
				Console.WriteLine("Usage: UltraFXR INFILE OUT.wav");
				Environment.Exit(1);
			}

			var p = ReadBfxr(args[0]);
			Console.WriteLine("BFXR Parameters:");
			p.Dump();

			WriteGenerator(args[1], new BfxrGenerator(p));
		}

		private static void WriteGenerator(string path, IGenerator generator)
		{
			float[] buffer = new float[1024];
			using (var s = new FileStream(path, FileMode.Create))
			using (var w = new WaveWriter(s, 44100))
			{
				bool done;
				do
				{
					done = generator.Render(buffer);
					w.Write(buffer);
				} while (!done);
			}
		}

		private static BfxrParameters ReadBfxr(string path)
		{
			string data;
			using (var r = new StreamReader(path, Encoding.ASCII))
			{
				data = r.ReadToEnd();
			}
			return BfxrParameters.Deserialize(data);
		}
	}
}
